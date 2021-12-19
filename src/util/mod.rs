#![allow(unused)]

/// Creates *const i8 from a &str
/// This either takes a literal and appends a null char (\0) to it.
/// or if it is an expression, tries to make a CString from it.
/// Will panic if passed an expression that a CString could not be created from.
/// # Examples
/// ```rust
/// use rglua::prelude::*;
/// let a = b"Hello world!".as_ptr() as *const i8;
/// let b = cstr!("Hello world!");
/// unsafe { assert_eq!(*a, *b) };
///
/// let c = "Hello world!";
/// let d = cstr!(c); // Macro doesn't know this is a literal, so it will try to make a CString
/// unsafe { assert_eq!(*b, *d.as_ptr()) };
/// ```
#[macro_export]
macro_rules! cstr {
	($rstring:literal) => {
		concat!($rstring, "\0").as_ptr() as *const i8
	};
	($rstring:expr) => {
		std::ffi::CString::new($rstring).expect("Couldn't make CString from rust string")
	};
}

/// Tries to create a *const i8 from a &str
/// This either takes a literal and appends a null char (\0) to it.
/// or if it is a value, makes a cstring and returns the pointer to it.
/// # Examples
/// ```rust, should_panic
/// use rglua::prelude::*;
/// let a = b"Hello world!".as_ptr() as *const i8;
/// let b = try_cstr!("Hello world!");
/// unsafe { assert_eq!(*a, *b) } ;
///
/// let c = "Invalid! 👎 \0"; // Cannot have nulls inside of it.
/// let d = try_cstr!(c).unwrap();
/// ```
#[macro_export]
macro_rules! try_cstr {
	($rstring:literal) => {
		concat!($rstring, "\0").as_ptr() as *const i8
	};
	($rstring:expr) => {{
		std::ffi::CString::new($rstring)
	}};
}

/// Tries to convert a const char* to a &str
/// Will panic if the const char* is not valid utf-8
/// # Examples
/// ```rust
/// use rglua::prelude::*;
/// let cstr = cstr!("Hello World");
/// let rust_str = rstr!(cstr);
/// assert_eq!(rust_str, "Hello World");
/// ```
#[macro_export]
macro_rules! rstr {
	($cstring:expr) => {{
		#[allow(unused_unsafe)]
		let cstr = unsafe { std::ffi::CStr::from_ptr($cstring) };
		cstr.to_str().expect("Couldn't unwrap CString")
	}};
}

#[macro_export]
/// Tries to convert a const char* to an &str
/// # Examples
/// ```rust
/// use rglua::prelude::*;
/// let cstr = cstr!("Hello World");
/// let rstr = try_rstr!(cstr);
/// assert!(rstr.is_ok()); // Should be perfectly valid to convert to utf8
/// ```
macro_rules! try_rstr {
	($cstring:expr) => {{
		#[allow(unused_unsafe)]
		let cstr = unsafe { std::ffi::CStr::from_ptr($cstring) };
		cstr.to_str()
	}};
}

#[allow(unused_macros)]
#[macro_export]
/// Like println!, however it prints to the gmod server's console.
/// First arg is the lua state.
/// Rest are varargs.
/// Can be either a variable storing a str literal, or a referenced String / str variable
/// # Examples
/// ```rust
/// use rglua::prelude::*;
/// fn gmod13_open(l: LuaState) {
///     let world = "world";
///     printgm!(l, "Hello {}!", world);
/// }
/// ```
macro_rules! printgm {
	($state:expr, $($x:expr),*) => {
		{
			let printargs = format!( $($x,)* );
			if let Ok(fmt) = std::ffi::CString::new(printargs) {
				rglua::lua::lua_getglobal( $state, rglua::cstr!("print") );
				rglua::lua::lua_pushstring( $state, fmt.as_ptr() );
				rglua::lua::lua_call( $state, 1, 0 );
			}
		}
	};
}

/// Creates an array of LuaRegs for you to be used with luaL_register
/// # Examples
/// Basic usage
/// ```rust
/// use rglua::prelude::*;
/// extern "C" fn max(l: LuaState) -> i32 { 0 }
/// extern "C" fn min(l: LuaState) -> i32 { 0 }
/// let my_library = reg! [
///     "max" => max,
///     "min" => min
/// ];
/// assert_eq!(my_library.len(), 3); // 2 functions + 1 internal null terminator
/// unsafe { assert_eq!(my_library[0].name, cstr!("max")) }; // Internally this is turned into &[ LuaReg { name: cstr!("max"), func: max }, ... ];
/// ```
/// Returns a &[crate::types::LuaReg]
#[macro_export]
macro_rules! reg {
	( $( $name:expr => $func:expr ),* ) => {
		&[ $( rglua::types::LuaReg { name: rglua::cstr!($name), func: Some($func) } ),*, rglua::types::LuaReg { name: std::ptr::null(), func: None } ]
	};
}

use crate::types::LuaState;
/// Returns the current state of the lua stack without affecting it.
/// Comes out in this format:
/// ```text
/// [1] 'number' = 5000
/// [2] 'string' = "hello"
/// [3] 'table' = 0x213542
/// [4] 'function' = 0x138252
/// [5] 'nil' = nil
/// ```
pub fn dump_stack(l: LuaState) -> Result<String, std::fmt::Error> {
	use std::fmt::Write;

	use crate::lua::{Type, *};
	let mut buf = String::new();

	let top = lua_gettop(l);
	for i in 1..=top {
		write!(&mut buf, "[{}] '{}' = ", i, rstr!(luaL_typename(l, i)));
		match lua_type(l, i) {
			Type::Number => write!(&mut buf, "{}", lua_tonumber(l, i)),
			Type::String => write!(&mut buf, "{}", rstr!(lua_tostring(l, i))),
			Type::Bool => write!(
				&mut buf,
				"{}",
				if lua_toboolean(l, i) == 1 {
					"true"
				} else {
					"false"
				}
			),
			Type::Nil => write!(&mut buf, "nil"),
			Type::None => write!(&mut buf, "none"),
			_ => write!(&mut buf, "{:p}", lua_topointer(l, i)),
		}?
	}

	Ok(buf)
}
