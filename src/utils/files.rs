use std::io::{self, Error};

use chrono::NaiveDate;

use crate::{
    CommandContext,
    models::dayfile::DayFile,
    store::{day_store::DayStore, fsday_store::FsDayStore},
    utils::dates::todays_date,
};

pub fn load_or_empty(ctx: &CommandContext, date: Option<NaiveDate>) -> Result<DayFile, Error> {
    let date = date.unwrap_or(todays_date());
    let store = FsDayStore::new(ctx.data_dir.clone(), ctx.vault.as_deref())?;

    match store.load(date) {
        Ok(df) => Ok(df),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(DayFile::new(date)),
        Err(e) => Err(e.into()),
    }
}

pub fn save_dayfile(ctx: &CommandContext, df: &DayFile) -> Result<(), Error> {
    let store = FsDayStore::new(ctx.data_dir.clone(), ctx.vault.as_deref())?;
    store.save(df)
}
