use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, ValueEnum, Copy)]
pub enum ListScope {
    Day,
    Focus,
    All,
}