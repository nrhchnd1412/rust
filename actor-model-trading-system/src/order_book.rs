
use ractor::{ActorRef, Actor, ActorProcessingErr, SupervisionEvent, RpcReplyPort};
use std::time::Duration;
use tokio::time::sleep;
use crate::market_data;
use crate::risk_manager;
use crate::risk_manager::{RiskManager, RiskMessage};

pub enum OrderBookMessage{
    UpdatePrice(market_data::MarketDataTick),
    GetBestBid(ractor::RpcReplyPort<Option<f64>>),
    GetBestAsk(ractor::RpcReplyPort<Option<f64>>),
    PlaceOrder(Order),
    UpdateRiskManager(ActorRef<risk_manager::RiskMessage>),
}

#[derive(Debug,Clone)]
pub struct Order{
    pub order_id: String,
    pub price: f64,
    pub symbol: String,
    pub side:OrderSide,
    pub quantity: u64,
}

#[derive(Debug,Clone)]
pub enum OrderSide{
    Buy,
    Sell,
}

pub struct OrderBook {
    pub(crate) symbol: String,
}

pub struct OrderBookState{
    symbol: String,
    best_bid:Option<f64>,
    best_ask:Option<f64>,
    last_price:Option<f64>,
    update_count:u64,
    risk_manager: Option<ActorRef<risk_manager::RiskMessage>>,
}

#[ractor::async_trait]
impl Actor for OrderBook {
    type Msg = OrderBookMessage;
    type State = OrderBookState;
    type Arguments = (String, Option<ActorRef<risk_manager::RiskMessage>>);

    async fn pre_start(&self, myself: ActorRef<Self::Msg>, (symbol,risk_manager): Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        println!("[Orderbook] initializing for {}",symbol);
        Ok(OrderBookState{
            symbol,
            best_ask:None,
            best_bid:None,
            last_price:None,
            update_count:0,
            risk_manager,
        })
    }

    async fn handle(&self, myself: ActorRef<Self::Msg>, message: Self::Msg, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match message{
            OrderBookMessage::UpdatePrice(tick) => {
                state.update_count += 1;
                state.last_price=Some(tick.price);
                
                // simulate spread
                state.best_bid=Some(tick.price-0.1);
                state.best_ask=Some(tick.price+0.1);
                
                if state.update_count%5==0{
                    println!("[Orderbook] {} updatesL last={:.2}, bid={:.2},ask ={:.2}",
                        state.update_count,
                        tick.price,
                        state.best_bid.unwrap(),
                        state.best_ask.unwrap()
                    );
                    
                }
            }
            OrderBookMessage::GetBestBid(reply) => {
                reply.send(state.best_bid)?;
            }
            OrderBookMessage::GetBestAsk(reply) => {
                reply.send(state.best_ask)?;
            }
            OrderBookMessage::PlaceOrder(order) => {
                if let Some(ref risk_manager)=state.risk_manager{
                    println!("[Orderbook] forwarding order {} to risk check",order.order_id);
                    risk_manager.cast(RiskMessage::CheckOrder(order))?;
                }
                else {
                    println!("[Orderbook] no risk manager to forward");
                }
            }
            OrderBookMessage::UpdateRiskManager(new_risk_manager) => {
                println!("[Orderbook] updating risk manager");
                state.risk_manager=Some(new_risk_manager);
            }
        }
        Ok(())
    }
}