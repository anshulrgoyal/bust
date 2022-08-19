use crate::request::Stats;
use prettytable::{Cell, Row, Table};

pub fn create_task_table(min: &Stats, max: &Stats, ac: &Stats, lookup_time: u128) {
    let mut table = Table::new();

    table.add_row(Row::new(vec![
        Cell::new("Task"),
        Cell::new("Min Time(milliseconds)"),
        Cell::new("Average Time(milliseconds)"),
        Cell::new("Max Time(milliseconds)"),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Dns Query"),
        Cell::new(&lookup_time.to_string()),
        Cell::new(&lookup_time.to_string()),
        Cell::new(&lookup_time.to_string()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Connection Time"),
        Cell::new(&min.connect.to_string()),
        Cell::new(&ac.connect.to_string()),
        Cell::new(&max.connect.to_string()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Tls Handshake Time"),
        Cell::new(&min.handshake.to_string()),
        Cell::new(&ac.handshake.to_string()),
        Cell::new(&max.handshake.to_string()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Waiting For Response"),
        Cell::new(&min.waiting.to_string()),
        Cell::new(&ac.waiting.to_string()),
        Cell::new(&max.waiting.to_string()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Writing the Request"),
        Cell::new(&min.writing.to_string()),
        Cell::new(&ac.writing.to_string()),
        Cell::new(&max.writing.to_string()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Compelete"),
        Cell::new(&min.compelete.to_string()),
        Cell::new(&ac.compelete.to_string()),
        Cell::new(&max.compelete.to_string()),
    ]));
    table.printstd();
}

pub fn create_percent_table(compeleted: &[u128]) {
    let total = compeleted.len();
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Percentage of Request"),
        Cell::new("Time(milliseconds)"),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("50%"),
        Cell::new(&compeleted[total / 2].to_string()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("75%"),
        Cell::new(&compeleted[total * 3 / 4].to_string()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("90%"),
        Cell::new(&compeleted[total * 9 / 10].to_string()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("95%"),
        Cell::new(&compeleted[total * 95 / 100].to_string()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("100%"),
        Cell::new(&compeleted[total - 1].to_string()),
    ]));
    table.printstd();
}
