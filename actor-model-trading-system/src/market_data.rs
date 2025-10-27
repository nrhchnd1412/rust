
use ractor::{ActorRef, Actor, ActorProcessingErr, SupervisionEvent};
use std::time::Duration;
use tokio::time::sleep;
use crate::order_book;

#[derive(Debug,Clone)]
pub struct MarketDataTick{
    pub symbol: String,
    pub price: f64,
    pub volume: u64,
    pub timestamp: u64,
}

pub enum MarketDataMessage{
    Start,
    Stop,
    Subscribe(ActorRef<order_book::OrderBookMessage>),
}

pub struct MartketDataFeed{
    pub symbol: String,
}

pub struct MarketDataState{
    symbol: String,
    subscribers: Vec<ActorRef<order_book::OrderBookMessage>>,
    connection_attempt: u32,
    is_running: bool,
    skip_first_failure: bool,
}

#[ractor::async_trait]
impl Actor for MartketDataFeed{
    type Msg = MarketDataMessage;
    type State = MarketDataState;
    type Arguments = (String,bool);

    async fn pre_start(&self, myself: ActorRef<Self::Msg>, (symbol, skip_first_failure): Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        println!("MarketDataFeed symbol: {}", symbol);
        Ok(MarketDataState{
            symbol,
            connection_attempt: 0,
            subscribers:Vec::new(),
            is_running: false,
            skip_first_failure
        })
    }

    async fn handle(&self, myself: ActorRef<Self::Msg>, message: Self::Msg, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match message{
            MarketDataMessage::Start => {
                state.connection_attempt += 1;
                state.is_running = true;

                //simulate connection failure on first attempt

                if state.connection_attempt ==1 && !state.skip_first_failure {
                    return Err(ActorProcessingErr::from("Failed to connect to exchange"));
                }
                println!("[MarketData] connected to exchange for symbol: {}", state.symbol);
                //start streaming data
                let symbol = state.symbol.clone();
                let subscribers = state.subscribers.clone();
                tokio::spawn(async move {
                    for i in 0..10{
                        let tick=MarketDataTick{
                            symbol: symbol.clone(),
                            price: 100.0 +(i as f64 * 0.5),
                            volume: 1000 +(i*100),
                            timestamp: i as u64,
                        };
                        for subscriber in &subscribers {
                            let _ = subscriber.cast(order_book::OrderBookMessage::UpdatePrice(tick.clone()));
                        }
                        sleep(Duration::from_millis(1)).await;
                    }
                });
            }
            MarketDataMessage::Stop => {
                state.is_running = false;
                println!("MarketDataFeed stopped with symbol: {}", state.symbol);
            }
            MarketDataMessage::Subscribe(subscriber) => {
                println!("MarketDataFeed new subscriber for : {}", state.symbol);
                state.subscribers.push(subscriber);
            }
        }
        Ok(())
    }

    async fn post_stop(&self, myself: ActorRef<Self::Msg>, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        println!("[MarketData] stopped after {} connection attempts", state.connection_attempt);
        Ok(())
    }
}
