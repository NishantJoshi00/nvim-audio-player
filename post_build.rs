use std::env;
fn main() {
    let home = env::var("HOME").expect("No home to go");
    let paths_path = format!("{}/.config/nvim/custom/player/lua", home);
    let path = std::path::Path::new(&paths_path);
    if !path.exists() {
        let player_dir = format!("{}/.config/nvim/custom/player/lua", home);
        std::fs::create_dir_all(&player_dir).unwrap();
    }
    let source = std::path::Path::new(&env::var("CRATE_OUT_DIR").unwrap()).join("libplayer.so");
    let dst_str = format!("{}/.config/nvim/custom/player/lua/player.so", home);
    let dst = std::path::Path::new(&dst_str);
    std::fs::copy(source, dst).unwrap();

}
