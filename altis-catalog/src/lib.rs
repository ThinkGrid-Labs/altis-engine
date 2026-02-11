pub mod product;
pub mod pricing;
pub mod inventory;

pub use product::{Product, ProductType, ProductTrait};
pub use pricing::{PricingContext, PricingEngine};
pub use inventory::InventoryManager;
