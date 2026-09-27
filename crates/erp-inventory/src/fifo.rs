/// FIFO valuation over contiguous buffers (Stage 4.3.1-4.3.2).
#[derive(Debug, Clone)] pub struct Layer { pub qty: i64, pub rate_cents: i64 }
#[derive(Default)] pub struct FifoQueue { pub layers: Vec<Layer> }
impl FifoQueue {
    pub fn purchase(&mut self, qty: i64, rate_cents: i64) { self.layers.push(Layer{qty, rate_cents}); }
    /// Consume qty, return COGS in cents.
    pub fn consume(&mut self, mut qty: i64) -> i64 {
        let mut cogs = 0;
        while qty > 0 && !self.layers.is_empty() {
            let take = qty.min(self.layers[0].qty);
            cogs += take * self.layers[0].rate_cents;
            self.layers[0].qty -= take; qty -= take;
            if self.layers[0].qty == 0 { self.layers.remove(0); }
        } cogs
    }
    pub fn balance_qty(&self) -> i64 { self.layers.iter().map(|l| l.qty).sum() }
}
#[cfg(test)] mod t { use super::*;
    #[test] fn fifo_cogs(){ let mut q = FifoQueue::default(); q.purchase(10,100); q.purchase(10,200);
        assert_eq!(q.consume(15), 10*100+5*200); assert_eq!(q.balance_qty(),5); }
}
