extern crate bindgen;

use std::path::PathBuf;
use std::process::Command;
use std::{env, io};

use bindgen::callbacks::{IntKind, ParseCallbacks, Token};

#[derive(Debug)]
struct CustomCallbacks;

impl ParseCallbacks for CustomCallbacks {
    // older versions of libsnmp break bindgen with casts to unsigned char
    // fixed in this commit https://github.com/net-snmp/net-snmp/commit/31a2c8065db102c876937ff64ee90c60b516d945
    fn modify_macro(&self, name: &str, tokens: &mut Vec<Token>) {
        if name.starts_with("ASN_") {
            tokens.retain(|t| {
                t.raw.as_ref() != [b'('] && t.raw.as_ref() != [b')'] && t.raw.as_ref() != b"u_char"
            });
        }
    }

    // fixup int types
    fn int_macro(&self, name: &str, value: i64) -> Option<IntKind> {
        match name {
            "SNMP_NOSUCHOBJECT" | "SNMP_NOSUCHINSTANCE" | "SNMP_ENDOFMIBVIEW" => {
                Some(IntKind::Custom {
                    name: "::core::ffi::c_uchar",
                    is_signed: false,
                })
            }
            _s if name.starts_with("ASN_") => Some(IntKind::Custom {
                name: "::core::ffi::c_uchar",
                is_signed: false,
            }),
            _s if name.starts_with("SNMP_VERSION") => Some(IntKind::Custom {
                name: "::core::ffi::c_long",
                is_signed: true,
            }),
            _s if name.starts_with("SNMP_ERR_") => Some(IntKind::Custom {
                name: "::core::ffi::c_long",
                is_signed: true,
            }),
            _ => {
                if value > i32::MAX as i64 || value < i32::MIN as i64 {
                    Some(IntKind::Custom {
                        name: "i64",
                        is_signed: true,
                    })
                } else {
                    Some(IntKind::Custom {
                        name: "::core::ffi::c_int",
                        is_signed: true,
                    })
                }
            }
        }
    }
}

fn main() {
    let libs = exec("net-snmp-config", ["--libs"]).expect("Unable to execute 'net-snmp-config'");

    println!("cargo::rustc-flags={}", parse_libs(&libs));

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .generate_cstr(true)
        .parse_callbacks(Box::new(CustomCallbacks))
        .allowlist_var("^.*(ASN|snmp|SNMP|NETSNMP|MIB|OID|STAT|VACM|MAX|MIN).*")
        .allowlist_function(
            ".*(snmp|netsnmp|SNMP|NETSNMP|init|shutdown|add|register|mib|free|value|variable|objid).*",
        )
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .use_core()
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn exec<I, S>(program: S, args: I) -> io::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let output = Command::new(program).args(args).output()?;
    if !output.status.success() {
        return Err(io::Error::other(String::from_utf8(output.stderr).unwrap()));
    }
    Ok(String::from_utf8(output.stdout).unwrap())
}

fn parse_libs(s: &str) -> String {
    let mut v = Vec::new();
    for part in s.split_ascii_whitespace() {
        if part.starts_with("-l") || part.starts_with("-L") {
            v.push(part);
        }
    }
    v.sort_unstable();
    v.dedup();
    v.join(" ")
}
