use crate::request::Stats;
use std::ops::Add;

pub fn calculate_stats(min: &mut Stats, max: &mut Stats, c: &Stats, ac: &mut Stats) {
    min.connect = std::cmp::min(min.connect, c.connect);
    min.handshake = std::cmp::min(min.handshake, c.handshake);
    min.waiting = std::cmp::min(min.waiting, c.waiting);
    min.writing = std::cmp::min(min.writing, c.writing);
    min.read = std::cmp::min(min.read, c.read);
    min.compelete = std::cmp::min(min.compelete, c.compelete);

    max.connect = std::cmp::max(max.connect, c.connect);
    max.handshake = std::cmp::max(max.handshake, c.handshake);
    max.waiting = std::cmp::max(max.waiting, c.waiting);
    max.writing = std::cmp::max(max.writing, c.writing);
    max.read = std::cmp::max(max.read, c.read);
    max.compelete = std::cmp::max(max.compelete, c.compelete);

    ac.connect = ac.connect.add(c.connect);
    ac.handshake = ac.handshake.add(c.handshake);
    ac.waiting = ac.waiting.add(c.waiting);
    ac.writing = ac.writing.add(c.writing);
    ac.read = ac.read.add(c.read);
    ac.compelete = ac.compelete.add(c.compelete);
}
