use std::fs::File;
use std::io;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::Ticket;
use crate::slot::LockDir;
use crate::slot::scratch::{Scratch, eventually};
use crate::units::{Name, parse_name};

fn key(s: &str) -> io::Result<Name> {
    parse_name(s).map_err(|e| io::Error::other(e.to_string()))
}

fn take(dir: &LockDir, k: &Name) -> io::Result<Ticket> {
    Ticket::take(dir, k).map_err(io::Error::other)
}

fn first(ticket: &Ticket) -> io::Result<bool> {
    ticket.is_first().map_err(io::Error::other)
}

#[test]
fn tickets_count_up_and_the_lowest_goes_first() -> io::Result<()> {
    let scratch = Scratch::new("fifo-order")?;
    let dir = scratch.lock_dir()?;
    let k = key("make")?;
    let one = take(&dir, &k)?;
    let two = take(&dir, &k)?;
    assert!(one.seq() < two.seq());
    assert!(first(&one)?);
    assert!(!first(&two)?);
    drop(one);
    assert!(eventually(|| first(&two), |f| *f)?);
    Ok(())
}

#[test]
fn a_dropped_ticket_leaves_no_file() -> io::Result<()> {
    let scratch = Scratch::new("fifo-drop")?;
    let dir = scratch.lock_dir()?;
    let ticket = take(&dir, &key("make")?)?;
    let name = format!("make+ticket.{}", ticket.seq());
    assert!(scratch.path().join(&name).is_file());
    drop(ticket);
    assert!(!scratch.path().join(&name).exists());
    Ok(())
}

#[test]
fn a_dead_lower_ticket_is_skipped_and_removed() -> io::Result<()> {
    let scratch = Scratch::new("fifo-dead")?;
    let dir = scratch.lock_dir()?;
    let k = key("make")?;
    let mine = take(&dir, &k)?;
    let dead = scratch.path().join("make+ticket.0");
    if mine.seq() == 0 {
        return Err(io::Error::other("expected a ticket past 0"));
    }
    File::create(&dead)?;
    assert!(first(&mine)?);
    assert!(!dead.exists());
    Ok(())
}

#[test]
fn other_keys_do_not_queue_together() -> io::Result<()> {
    let scratch = Scratch::new("fifo-keys")?;
    let dir = scratch.lock_dir()?;
    let a = take(&dir, &key("a")?)?;
    let b = take(&dir, &key("b")?)?;
    assert!(first(&a)? && first(&b)?);
    Ok(())
}

/// Spins until `ticket` is first, then records its number.
fn wait_turn(ticket: Ticket, order: &Mutex<Vec<u64>>) -> io::Result<()> {
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(2))
        .ok_or_else(|| io::Error::other("clock"))?;
    while !first(&ticket)? {
        if Instant::now() > deadline {
            return Err(io::Error::other("ticket never came first"));
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    order
        .lock()
        .map_err(|_| io::Error::other("poison"))?
        .push(ticket.seq());
    Ok(())
}

/// Every ticket waits for its turn in a thread of its own, the last
/// taken started first; the numbers in the order they went.
fn race(tickets: Vec<Ticket>) -> io::Result<Vec<u64>> {
    let order = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        let handles: Vec<_> = tickets
            .into_iter()
            .rev()
            .map(|t| s.spawn(|| wait_turn(t, &order)))
            .collect();
        handles.into_iter().try_for_each(|h| {
            h.join().map_err(|_| io::Error::other("thread"))?
        })
    })?;
    order.into_inner().map_err(|_| io::Error::other("poison"))
}

#[test]
fn waiters_in_threads_go_in_ticket_order() -> io::Result<()> {
    let scratch = Scratch::new("fifo-threads")?;
    let dir = scratch.lock_dir()?;
    let k = key("make")?;
    let tickets = (0..4)
        .map(|_| take(&dir, &k))
        .collect::<io::Result<Vec<_>>>()?;
    let want: Vec<u64> = tickets.iter().map(Ticket::seq).collect();
    assert_eq!(race(tickets)?, want);
    Ok(())
}
