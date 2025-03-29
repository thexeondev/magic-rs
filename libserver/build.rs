fn main() {
    println!("cargo:rustc-link-search=native=C:\\android-ndk-r27c\\toolchains\\llvm\\prebuilt\\windows-x86_64\\lib\\clang\\18\\lib\\linux");
    println!("cargo:rustc-link-lib=static=clang_rt.builtins-i686-android.a");
}
