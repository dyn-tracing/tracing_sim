fn main() {
  let dyn_lib   = libloading::Library::new("target/debug/libplugin_sample.dylib").expect("load library");
  let loaded_fn : libloading::Symbol<extern fn(u32) -> ()> = unsafe { dyn_lib.get(b"codelet") }.expect("load symbol");
  loaded_fn(53);
}
