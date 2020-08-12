// TODO tables https://docs.rs/diesel/1.4.5/diesel/macro.table.html
// https://github.com/diesel-rs/diesel/blob/master/examples/mysql/getting_started_step_1/src/schema.rs

struct Database {
    server: String,
    port: u16
}

impl Database {
    pub fn init() {

    }
    pub fn conn() {
        // TODO
    }
}