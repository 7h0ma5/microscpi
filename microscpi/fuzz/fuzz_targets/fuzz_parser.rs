#![no_main]

use libfuzzer_sys::fuzz_target;
use microscpi::Node;

static ROOT_NODE: Node = Node {
    children: &[("*IDN", &IDN_NODE), ("SYST", &SYST_NODE)],
    command: None,
    query: None,
};

static IDN_NODE: Node = Node {
    children: &[],
    command: None,
    query: None,
};

static SYST_NODE: Node = Node {
    children: &[("ERR", &ERR_NODE)],
    command: None,
    query: None,
};

static ERR_NODE: Node = Node {
    children: &[],
    command: None,
    query: None,
};

fuzz_target!(|data: &[u8]| {
    let _ = microscpi::parser::parse(&ROOT_NODE, data);
});
