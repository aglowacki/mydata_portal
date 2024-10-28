mod dm_api;
mod redis_api;


fn main() 
{
    dm_api::get_session_id().ok();
    //println!("Hello, world!");
}

