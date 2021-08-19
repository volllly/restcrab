macro_rules! ok_or_push {
  ($wrapped:expr, $push:ident, $on_err:expr) => {
    match $wrapped {
      Ok(ok) => ok,
      Err(err) => {
        $push.push(err);
        $on_err
      }
    }
  };
}
