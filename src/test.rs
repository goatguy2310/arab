use tetr_ch::client::{
    Client,
    stream::{StreamType, StreamContext}
};
use tetr_ch::model::{user::{UserRecordsResponse, UserResponse, User, FortyLines, Blitz}, league::Rank};

// 5f9d3503f05507df724e6a54
#[test]
fn main() {
    let lrs = Client::new().get_stream(StreamType::League, StreamContext::UserRecent, Some("5f9d3503f05507df724e6a54")).await.unwrap();
    let vec = &lrs.data.as_ref().unwrap().records;
    println!("{}", vec.size());
}
