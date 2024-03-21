#[cfg(debug_assertions)]
use dotenvy::dotenv;

pub fn load() {
    #[cfg(debug_assertions)]
    {
        dotenv().expect(".envファイルが存在しません");
    }
}
