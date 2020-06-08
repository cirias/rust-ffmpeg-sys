extern crate bindgen;
extern crate regex;

use std::env;
use std::path::PathBuf;

use regex::Regex;
use bindgen::callbacks::{IntKind, ParseCallbacks, MacroParsingBehavior};

fn main() {
    println!("cargo:rustc-link-search=native={}", ffmpeg_static_path().join("lib").to_string_lossy());
    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    println!("cargo:rustc-link-lib=static=avformat");
    println!("cargo:rustc-link-lib=static=avcodec");
    println!("cargo:rustc-link-lib=static=avutil");
    println!("cargo:rustc-link-lib=static=swscale");
    println!("cargo:rustc-link-lib=static=swresample");
    println!("cargo:rustc-link-lib=static=xml2");
    println!("cargo:rustc-link-lib=static=ssl");
    println!("cargo:rustc-link-lib=static=crypto");
    println!("cargo:rustc-link-lib=static=z");
    println!("cargo:rustc-link-lib=static=x264");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    // println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .ctypes_prefix("libc")
        // https://github.com/servo/rust-bindgen/issues/550
        .blacklist_type("max_align_t")
        .rustified_enum("*")
        .prepend_enum_name(false)
        .derive_eq(true)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        // .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .parse_callbacks(Box::new(IntCallbacks))
        // The input header we would like to generate
        // bindings for.
        .header(ffmpeg_static_path().join("include/libavformat/avformat.h").to_string_lossy())
        .header(ffmpeg_static_path().join("include/libavdevice/avdevice.h").to_string_lossy())
        .header(ffmpeg_static_path().join("include/libavcodec/avcodec.h").to_string_lossy())
        .header(ffmpeg_static_path().join("include/libavfilter/avfilter.h").to_string_lossy())
        .header(ffmpeg_static_path().join("include/libavutil/avutil.h").to_string_lossy())
        .header(ffmpeg_static_path().join("include/libavutil/imgutils.h").to_string_lossy())
        .header(ffmpeg_static_path().join("include/libavutil/pixfmt.h").to_string_lossy())
        .header(ffmpeg_static_path().join("include/libavutil/time.h").to_string_lossy())
        .header(ffmpeg_static_path().join("include/libswscale/swscale.h").to_string_lossy())
        .header(ffmpeg_static_path().join("include/libavfilter/buffersink.h").to_string_lossy())
        .header(ffmpeg_static_path().join("include/libavfilter/buffersrc.h").to_string_lossy())


        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

#[derive(Debug)]
struct IntCallbacks;

impl ParseCallbacks for IntCallbacks {
    fn int_macro(&self, _name: &str, value: i64) -> Option<IntKind> {
        let ch_layout = Regex::new(r"^AV_CH").unwrap();
        let codec_cap = Regex::new(r"^AV_CODEC_CAP").unwrap();
        let codec_flag = Regex::new(r"^AV_CODEC_FLAG").unwrap();
        let error_max_size = Regex::new(r"^AV_ERROR_MAX_STRING_SIZE").unwrap();

        if value >= i64::min_value() as i64 && value <= i64::max_value() as i64
            && ch_layout.is_match(_name)
        {
            Some(IntKind::ULongLong)
        } else if value >= i32::min_value() as i64 && value <= i32::max_value() as i64
            && (codec_cap.is_match(_name) || codec_flag.is_match(_name))
        {
            Some(IntKind::UInt)
        } else if error_max_size.is_match(_name) {
            Some(IntKind::Custom {
                name: "usize",
                is_signed: false,
            })
        } else if value >= i32::min_value() as i64 && value <= i32::max_value() as i64 {
            Some(IntKind::Int)
        } else {
            None
        }
    }

    // https://github.com/rust-lang/rust-bindgen/issues/687#issuecomment-388277405
    fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
        use MacroParsingBehavior::*;

        match name {
            "FP_INFINITE" => Ignore,
            "FP_NAN" => Ignore,
            "FP_NORMAL" => Ignore,
            "FP_SUBNORMAL" => Ignore,
            "FP_ZERO" => Ignore,
            _ => Default,
        }
    }
}

fn ffmpeg_static_path() -> PathBuf {
    env::current_dir()
        .unwrap()
        .join("ffmpeg-static/ffmpeg-lite/debian-9-output/ffmpeg")
}

/*
 * fn search_include(include_paths: &Vec<PathBuf>, header: &str) -> String {
 *     for dir in include_paths {
 *         let include = dir.join(header);
 *         if fs::metadata(&include).is_ok() {
 *             return format!("{}", include.as_path().to_str().unwrap());
 *         }
 *     }
 *     format!("/usr/include/{}", header)
 * }
 */
