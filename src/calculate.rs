use crate::request::Stats;
use std::cmp::{min,max};

pub fn calculate_stats(min_value: &mut Stats, max_value: &mut Stats, c: &Stats, ac: &mut Stats) {
    min_value.connect = min(min_value.connect, c.connect);
    min_value.handshake = min(min_value.handshake, c.handshake);
    min_value.waiting = min(min_value.waiting, c.waiting);
    min_value.writing = min(min_value.writing, c.writing);
    min_value.read = min(min_value.read, c.read);
    min_value.compelete = min(min_value.compelete, c.compelete);

    max_value.connect = max(max_value.connect, c.connect);
    max_value.handshake = max(max_value.handshake, c.handshake);
    max_value.waiting = max(max_value.waiting, c.waiting);
    max_value.writing = max(max_value.writing, c.writing);
    max_value.read = max(max_value.read, c.read);
    max_value.compelete = max(max_value.compelete, c.compelete);

    ac.connect += c.connect;
    ac.handshake += c.handshake;
    ac.waiting += c.waiting;
    ac.writing += c.writing;
    ac.read += c.read;
    ac.compelete += c.compelete;
}
