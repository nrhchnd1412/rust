use ractor::{ActorRef, Actor, ActorProcessingErr, SupervisionEvent, RpcReplyPort};
use std::time::Duration;
use tokio::time::sleep;
use crate::executor::{ExecutorMessage, OrderExecutor};
use crate::market_data::{MarketDataMessage, MartketDataFeed};
use crate::order_book::{Order, OrderBook, OrderBookMessage};
use crate::risk_manager::{RiskManager, RiskMessage};

pub enum TradingSystemMessage{
    Start,
    SubmitOrder(Order),
    GetSystemStats(ractor::RpcReplyPort<SystemStats>),
    Shutdown,
}

#[derive(Debug,Clone)]
pub struct SystemStats{
    pub market_data_restarts: u32,
    pub risk_manager_restarts: u32,
    pub order_executed: u64,
}

pub struct SupervisorState{
    symbol: String,
    market_data: Option<ActorRef<MarketDataMessage>>,
    order_book: Option<ActorRef<OrderBookMessage>>,
    risk_manager: Option<ActorRef<RiskMessage>>,
    executor: Option<ActorRef<ExecutorMessage>>,
    market_data_restarts: u32,
    risk_manager_restarts: u32,

}

pub struct TradingSystemSupervisor {
    symbol:String
}

#[ractor::async_trait]
impl Actor for TradingSystemSupervisor{
    type Msg = TradingSystemMessage;
    type State = SupervisorState;
    type Arguments = String;

    async fn pre_start(&self, myself: ActorRef<Self::Msg>, symbol: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        println!("[Supervisor] initializing trading system for {}", symbol);

        //spawn executor (no supervisor needed, Stateless)
        let (executor_ref,_)=Actor::spawn(
            Some(format!("{}-executor", symbol)),
            OrderExecutor,
            ()
        ).await?;


        //spawn risk m anager with supervision
        let (risk_ref,_)=Actor::spawn_linked(
            Some(format!("{}-risk", symbol)),
            RiskManager,
            (1000,Some(executor_ref.clone())),
            myself.clone().into(),
        ).await?;

        //spawn order book
        let (book_ref,_)=Actor::spawn(
            Some(format!("{}-book", symbol)),
            OrderBook{symbol:symbol.clone()},
            (symbol.clone(),Some(risk_ref.clone())),
        ).await?;

        //spawn marketdata with supervision (allow first failure)
        let (market_ref,_)=Actor::spawn_linked(
            Some(format!("{}-market", symbol)),
            MartketDataFeed{symbol:symbol.clone()},
            (symbol.clone(),false),
            myself.clone().into(),
        ).await?;

        Ok(SupervisorState{
            symbol,
            market_data:Some(market_ref),
            order_book: Some(book_ref),
            risk_manager:Some(risk_ref),
            executor:Some(executor_ref),
            market_data_restarts:0,
            risk_manager_restarts:0,
        })

    }
    async fn handle(&self, myself: ActorRef<Self::Msg>, message: Self::Msg, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match message{
            TradingSystemMessage::Start => {
                if let Some(ref market_data) = state.market_data{
                    println!("[Supervisor] Starting market data feed...]");
                    market_data.cast(MarketDataMessage::Start)?;
                }
            }
            TradingSystemMessage::SubmitOrder(order) => {
                if let Some(ref order_book) = state.order_book{
                    order_book.cast(OrderBookMessage::PlaceOrder(order))?;
                }
            }
            TradingSystemMessage::GetSystemStats(reply) => {
                let executed = if let Some(ref executor)=state.executor{
                    match executor.call(
                        |r|ExecutorMessage::GetExecutedCount(r),
                        Some(Duration::from_secs(1))
                    ).await{
                        Ok(ractor::rpc::CallResult::Success(count))=>count,
                        _=>0,
                    }
                }else{
                    0
                };
                reply.send(SystemStats{
                    market_data_restarts:state.market_data_restarts,
                    risk_manager_restarts:state.risk_manager_restarts,
                    order_executed:executed,
                })?;
            }
            TradingSystemMessage::Shutdown => {
                println!("[Supervisor] Shutting down trading system...");
            }
        }
        Ok(())
    }
}