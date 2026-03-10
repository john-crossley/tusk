use std::io::{self, Error};

use chrono::NaiveDate;

use crate::{
    CommandContext,
    models::{dayfile::DayFile, focus_file::FocusFile, item::Item},
    store::{
        day_store::DayStore,
        focus_store::FocusStore,
        fs::{day_store::FsDayStore, focus_store::FsFocusStore},
    },
    utils::dates::todays_date,
};


// TODO:
// Can we have a make store and then pass that around instead of making it each time?

pub fn load_day_or_empty(ctx: &CommandContext, date: Option<NaiveDate>) -> Result<DayFile, Error> {
    let date = date.unwrap_or(todays_date());
    let store = FsDayStore::new(ctx.data_dir.clone(), ctx.vault.as_deref())?;

    match store.load(date) {
        Ok(df) => Ok(df),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(DayFile::new(date)),
        Err(e) => Err(e.into()),
    }
}

pub fn load_focus_or_empty(ctx: &CommandContext) -> Result<FocusFile, Error> {
    let store = FsFocusStore::new(ctx.data_dir.clone(), ctx.vault.as_deref())?;

    match store.load() {
        Ok(ff) => Ok(ff),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(FocusFile::new()),
        Err(e) => Err(e.into()),
    }
}

pub fn save_dayfile(ctx: &CommandContext, df: &DayFile) -> Result<(), Error> {
    let store = FsDayStore::new(ctx.data_dir.clone(), ctx.vault.as_deref())?;
    store.save(df)
}

pub fn save_focusfile(ctx: &CommandContext, ff: &FocusFile) -> Result<(), Error> {
    let store = FsFocusStore::new(ctx.data_dir.clone(), ctx.vault.as_deref())?;
    store.save(ff)
}