#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate glium;
extern crate image as image_ext;
extern crate libc;
extern crate processing as p5;
extern crate strobe;

use libc::size_t;
use std::os::raw::c_char;
use std::os::raw::c_double;
use std::os::raw::c_int;
use std::os::raw::c_void;

macro_rules! get_string {
    ( $prhs:ident, $x:expr, $func_name:expr ) => {{
        let mut ret = 20;
        let mut c_buf: [c_char; 100] = [0; 100];
        let mut s = "";
        unsafe {
            if !mxIsChar(*($prhs.offset($x)) as *const mxArray) {
                genMatlabErrorMsg!(
                    format!("Strobe:{}", $func_name),
                    format!("Please make sure that argument #{} is a valid string.", $x)
                );
            } else {
                ret = mxGetString(
                    *($prhs.offset($x)) as *const mxArray,
                    c_buf[..].as_mut_ptr(),
                    99,
                );
                s = std::ffi::CStr::from_ptr(c_buf[..].as_ptr())
                    .to_str()
                    .unwrap();
            }
            s
        }
    }};
}

// maybe re-write this to avoid potential ffi problems with memory allocation
macro_rules! get_chars_as_vec_i8 {
    ( $prhs:ident, $x:expr, $func_name:expr ) => {{
        let mut ret = 20;
        let mut c_buf: [c_char; 100] = [0; 100];
        let mut i8_vec = vec![0i8];
        unsafe {
            if !mxIsChar(*($prhs.offset($x)) as *const mxArray) {
                genMatlabErrorMsg!(
                    format!("Strobe:{}", $func_name),
                    format!("Please make sure that argument #{} is a valid string.", $x)
                );
            } else {
                ret = mxGetString(
                    *($prhs.offset($x)) as *const mxArray,
                    c_buf[..].as_mut_ptr(),
                    99,
                );
                i8_vec = std::ffi::CStr::from_ptr(c_buf[..].as_ptr())
                    .to_str()
                    .unwrap()
                    .as_bytes()
                    .iter()
                    .map(|x| *x as i8)
                    .collect::<Vec<i8>>();
                i8_vec.push('\0' as i8);
            }
            i8_vec
        }
    }};
}

macro_rules! genMatlabErrorMsg {
    ( $errid:expr, $errmsg:expr ) => {
        mexErrMsgIdAndTxt(
            std::ffi::CString::new($errid)
                .expect("failed when generating error message")
                .as_ptr(),
            std::ffi::CString::new($errmsg)
                .expect("failed when generating error message")
                .as_ptr(),
        )
    };
}

macro_rules! is_it_floating_scalar {
    ( $x:expr ) => {
        mxIsDouble($x) && !mxIsComplex($x) && mxGetNumberOfElements($x) == 1
    };
}

macro_rules! is_it_u32_scalar {
    ( $x:expr ) => {
        mxGetClassID($x) == mxClassID_mxUINT32_CLASS
            && !mxIsComplex($x)
            && mxGetNumberOfElements($x) == 1
    };
}

macro_rules! is_it_u64_w_type {
    ( $x:expr ) => {
        mxGetClassID($x) == mxClassID_mxUINT64_CLASS
            && !mxIsComplex($x)
            && mxGetNumberOfElements($x) == 2
    };
}

macro_rules! is_it_floating_matrix {
    ( $x:expr ) => {
        mxIsDouble($x) && !mxIsComplex($x)
    };
}

// still need to check for the following:
// - vectors of elements (and start to adapt strobe-rust for them)
// - clean up windows ulong mishap
// - make better shader setting generic function

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mxArray_tag {
    _unused: [u8; 0],
}
type mxArray = mxArray_tag;

type mwSize = c_int;

type mxLogical = bool;
type mxInt8 = i8;
type mxUint8 = u8;
type mxInt16 = i16;
type mxUint16 = u16;
type mxInt32 = i32;
type mxUint32 = u32;
type mxInt64 = i64;
type mxUint64 = u64;
type mxSingle = f32;
type mxDouble = f64;

type mxClassID = i32;
const mxClassID_mxUINT32_CLASS: mxClassID = 13;
const mxClassID_mxUINT64_CLASS: mxClassID = 15;
type mxComplexity = i32;
const mxComplexity_mxREAL: mxComplexity = 0;

// strobe types - simple C-style enum
const StrobeScreen: u64 = 1;
const StrobeShape: u64 = 2;
const StrobeTexture: u64 = 3;
const StrobeFBO: u64 = 4;
const StrobeImage: u64 = 5;
const StrobeMould: u64 = 6;
const StrobeShader: u64 = 7;

#[link(name = "libmex")]
extern "C" {
    fn mexPrintf(fmt: *const u8, ...);
    fn mexErrMsgIdAndTxt(errorid: *const c_char, errormsg: *const c_char, ...);
}

#[link(name = "libmx")]
extern "C" {
    fn mxGetScalar(pa: *const mxArray) -> c_double;
    fn mxCreateDoubleScalar(value: c_double) -> *mut mxArray;
    fn mxGetLogicals(array_ptr: *const mxArray) -> *mut mxLogical;
    fn mxGetString(pa: *const mxArray, buf: *mut c_char, buflen: mwSize) -> c_int;
    fn mxGetClassID(pm: *const mxArray) -> mxClassID;
    // fn mxIsScalar(array_ptr: *const mxArray) -> mxLogical;
    fn mxIsLogicalScalar(array_ptr: *const mxArray) -> mxLogical;
    fn mxIsLogical(array_ptr: *const mxArray) -> mxLogical;
    fn mxIsInt8(pm: *const mxArray) -> mxLogical;
    fn mxIsUint8(pm: *const mxArray) -> mxLogical;
    fn mxIsInt16(pm: *const mxArray) -> mxLogical;
    fn mxIsUint16(pm: *const mxArray) -> mxLogical;
    fn mxIsUint32(pm: *const mxArray) -> mxLogical;
    fn mxIsInt32(pm: *const mxArray) -> mxLogical;
    fn mxIsInt64(pm: *const mxArray) -> mxLogical;
    fn mxIsUint64(pm: *const mxArray) -> mxLogical;
    fn mxIsChar(pm: *const mxArray) -> mxLogical;
    fn mxIsDouble(pm: *const mxArray) -> mxLogical;
    fn mxIsSingle(pm: *const mxArray) -> mxLogical;
    fn mxIsComplex(pm: *const mxArray) -> mxLogical;
    fn mxGetNumberOfElements(pm: *const mxArray) -> size_t;
    // need a better solution for the items below...
    fn mxGetDoubles_800(pa: *const mxArray) -> *mut mxDouble;
    fn mxGetSingles_800(pa: *const mxArray) -> *mut mxSingle;
    fn mxGetInt8s_800(pa: *const mxArray) -> *mut mxInt8;
    fn mxGetUint8s_800(pa: *const mxArray) -> *mut mxUint8;
    fn mxGetInt16s_800(pa: *const mxArray) -> *mut mxInt16;
    fn mxGetUint16s_800(pa: *const mxArray) -> *mut mxUint16;
    fn mxGetInt32s_800(pa: *const mxArray) -> *mut mxInt32;
    fn mxGetUint32s_800(pa: *const mxArray) -> *mut mxUint32;
    fn mxGetInt64s_800(pa: *const mxArray) -> *mut mxInt64;
    fn mxGetUint64s_800(pa: *const mxArray) -> *mut mxUint64;
}

#[link(name = "libmat")]
extern "C" {
    fn mxCreateNumericMatrix(
        m: mwSize,
        n: mwSize,
        classid: mxClassID,
        ComplexFlag: mxComplexity,
    ) -> *const mxArray;
    fn mxGetData(pm: *const mxArray) -> *const c_void;
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn mexFunction<'a>(
    nlhs: c_int,
    plhs: *mut *mut mxArray,
    nrhs: c_int,
    prhs: *mut *mut mxArray,
) {
    let fn_name = get_string!(prhs, 0, "strobeMainFunction");
    // unsafe {
    // mexPrintf((&fn_name).as_ptr());
    // }

    match fn_name {
        // color
        "background" => unsafe {
            if nrhs != 6 {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please provide the screen, whose background color should be changed,
                    as well as the 4 RGBA components for the new background color."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))
                && is_it_floating_scalar!(*(prhs.offset(3)))
                && is_it_floating_scalar!(*(prhs.offset(4)))
                && is_it_floating_scalar!(*(prhs.offset(5))))
            {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the R, G, B, and A components of the requested color
                    are all real numbers."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let r = mxGetScalar(*(prhs.offset(2))) as f32;
            let g = mxGetScalar(*(prhs.offset(3))) as f32;
            let b = mxGetScalar(*(prhs.offset(4))) as f32;
            let a = mxGetScalar(*(prhs.offset(5))) as f32;

            strobe::background(scr, r, g, b, a);
        },
        "color_mode" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:color_mode",
                    "Please provide the screen, whose color mode should be changed,
                    and the desired color mode."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:color_mode",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:color_mode",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            // type check is done in macro
            let mode = get_chars_as_vec_i8!(prhs, 2, "color_mode");

            strobe::color_mode(scr, (&mode).as_ptr());
        },
        "stroke" => unsafe {
            if nrhs != 6 {
                genMatlabErrorMsg!(
                    "Strobe:stroke",
                    "Please provide the screen, whose stroke color should be changed,
                    as well as the 4 RGBA components for the new stroke color."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:stroke",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:stroke",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))
                && is_it_floating_scalar!(*(prhs.offset(3)))
                && is_it_floating_scalar!(*(prhs.offset(4)))
                && is_it_floating_scalar!(*(prhs.offset(5))))
            {
                genMatlabErrorMsg!(
                    "Strobe:stroke",
                    "Please make sure that the R, G, B, and A components of the
                    requested color are all real numbers."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let r = mxGetScalar(*(prhs.offset(2))) as f32;
            let g = mxGetScalar(*(prhs.offset(3))) as f32;
            let b = mxGetScalar(*(prhs.offset(4))) as f32;
            let a = mxGetScalar(*(prhs.offset(5))) as f32;

            strobe::stroke(scr, r, g, b, a);
        },
        "stroke_on" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:stroke_on",
                    "Please provide a screen, for which you want to activate stroke drawing."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:stroke_on",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:stroke_on",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::stroke_on(scr);
        },
        "stroke_off" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:stroke_off",
                    "Please provide the screen, for which you want to de-activate stroke drawing."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:stroke_off",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:stroke_off",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::stroke_off(scr);
        },
        "fill" => unsafe {
            if nrhs != 6 {
                genMatlabErrorMsg!(
                    "Strobe:fill",
                    "Please provide the screen, whose fill color should be changed,
                    as well as the 4 RGBA components for the new fill color."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:fill",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:fill",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))
                && is_it_floating_scalar!(*(prhs.offset(3)))
                && is_it_floating_scalar!(*(prhs.offset(4)))
                && is_it_floating_scalar!(*(prhs.offset(5))))
            {
                genMatlabErrorMsg!(
                    "Strobe:fill",
                    "Please make sure that the R, G, B, and A components of the requested
                    color are all real numbers."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let r = mxGetScalar(*(prhs.offset(2))) as f32;
            let g = mxGetScalar(*(prhs.offset(3))) as f32;
            let b = mxGetScalar(*(prhs.offset(4))) as f32;
            let a = mxGetScalar(*(prhs.offset(5))) as f32;

            strobe::fill(scr, r, g, b, a);
        },
        "fill_on" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:fill_on",
                    "Please provide the screen, for which you want to activate fill
                    drawing."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:fill_on",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:fill_on",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::fill_on(scr);
        },
        "fill_off" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:fill_off",
                    "Please provide the screen, for which you want to de-activate fill
                    drawing."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:fill_off",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:fill_off",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::fill_off(scr);
        },

        // environment
        "frame_count" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:frame_count",
                    "Please provide the screen, whose frame count you would like to know."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:frame_count",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:frame_count",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let fc = strobe::frame_count(scr);

            let p = plhs.offset(0);
            *p = mxCreateDoubleScalar(fc as f64);
        },
        "set_frame_rate" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:set_frame_rate",
                    "Please provide the screen, whose frame rate you would like to change,
                    as well as the new frame rate."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:set_frame_rate",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:set_frame_rate",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:set_frame_rate",
                    "Please make sure that the second argument is a positive number (i.e., >=0)."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let f_rate = mxGetScalar(*(prhs.offset(2))) as i32;

            if f_rate < 0 {
                genMatlabErrorMsg!(
                    "Strobe:set_frame_rate",
                    "Please make sure that the second argument is a positive number (i.e., >=0)."
                );
            }

            strobe::set_frame_rate(scr, f_rate);
        },
        "get_frame_rate" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:get_frame_rate",
                    "Please provide the screen, whose frame rate you would like to know."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:get_frame_rate",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:get_frame_rate",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let fr = strobe::get_frame_rate(scr);

            let p = plhs.offset(0);
            *p = mxCreateDoubleScalar(fr as f64);
        },
        "height" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:height",
                    "Please provide the screen, whose height you would like to know."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:height",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:height",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let hei = strobe::height(scr);

            let p = plhs.offset(0);
            *p = mxCreateDoubleScalar(hei as f64);
        },
        "width" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:width",
                    "Please provide the screen, whose width you would like to know."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:width",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:width",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let wid = strobe::width(scr);

            let p = plhs.offset(0);
            *p = mxCreateDoubleScalar(wid as f64);
        },
        "smooth" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:smooth",
                    "Please provide the screen, for which you would like to activate smoothed
                    drawing."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:smooth",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:smooth",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::smooth(scr);
        },
        "no_smooth" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:no_smooth",
                    "Please provide the screen, for which you would like to de-activate smoothed
                    drawing."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:no_smooth",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:no_smooth",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::no_smooth(scr);
        },
        "no_cursor" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:no_cursor",
                    "Please provide the screen, whose cursor you would like to hide."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:no_cursor",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:no_cursor",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::no_cursor(scr);
        },
        "focused" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:focused",
                    "Please provide the screen that you would like to be brought to the
                    foreground and focused for input."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:focused",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:focused",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::focused(scr);
        },
        "cursor" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!("Strobe:cursor",
                    "Please provide the screen, for which you would like to re-enable display of the cursor,
                    as well as the particular cursor type you would like."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:cursor",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:cursor",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            // type check is done in macro
            let cursor_type = get_chars_as_vec_i8!(prhs, 2, "cursor");

            strobe::cursor(scr, (&cursor_type).as_ptr());
        },
        "reset_cursor" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:reset_cursor",
                    "Please provide the screen, whose default cursor settings (i.e., display
                    cursor with default OS mouse pointer icon) you would like to re-enable."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:reset_cursor",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:reset_cursor",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::reset_cursor(scr);
        },

        // framebuffer
        "framebuffer" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!("Strobe:framebuffer",
                    "Please provide the screen, whose framebuffer should have a texture attached to it.
                    Also, please provide the texture. All drawing to the requested framebuffer will
                    be copied to the texture, which can be later displayed on to the screen."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:framebuffer",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1))) && is_it_u64_w_type!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:framebuffer",
                    "Please make sure that the first argument is a valid screen
                    and that the second argument is a valid texture."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let p = *(prhs.offset(2)) as *const mxArray;
            let tex_addr = mxGetData(p) as *const u64;
            if *(tex_addr.offset(1)) != StrobeTexture {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the second argument is a valid texture."
                );
            }
            let tex = std::mem::transmute::<u64, *const glium::texture::Texture2d>(*tex_addr);

            let frame_buf = strobe::framebuffer(scr, tex) as u64;

            let fbAddr = mxCreateNumericMatrix(1, 2, mxClassID_mxUINT64_CLASS, mxComplexity_mxREAL);
            let matlab_addr = mxGetData(fbAddr) as *mut u64;
            *matlab_addr = frame_buf;
            let strobe_type_addr = matlab_addr.offset(1);
            *strobe_type_addr = StrobeFBO;
            let p = plhs.offset(0);
            *p = fbAddr as *mut mxArray;
        },
        "clear_framebuffer" => unsafe {
            if nrhs != 7 {
                genMatlabErrorMsg!("Strobe:clear_framebuffer",
                    "Please provide the screen, whose framebuffer should be cleared to a given color.
                    Also, please provide the associated framebuffer that should be cleared, as well
                    as the 4 RGBA components for the color."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:clear_framebuffer",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1))) && is_it_u64_w_type!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:clear_framebuffer",
                    "Please make sure that the first argument is a valid screen
                    and that the second argument is a valid framebuffer."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(3)))
                && is_it_floating_scalar!(*(prhs.offset(4)))
                && is_it_floating_scalar!(*(prhs.offset(5)))
                && is_it_floating_scalar!(*(prhs.offset(6))))
            {
                genMatlabErrorMsg!(
                    "Strobe:clear_framebuffer",
                    "Please make sure that the R, G, B, and A components of the requested
                    color are all real numbers."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let p = *(prhs.offset(2)) as *const mxArray;
            let fbo_ptr_addr = mxGetData(p) as *const u64;
            if *(fbo_ptr_addr.offset(1)) != StrobeFBO {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the second argument is a valid framebuffer."
                );
            }
            let fbo_ptr = std::mem::transmute::<u64, *mut glium::framebuffer::SimpleFrameBuffer>(
                *fbo_ptr_addr,
            );

            let r = mxGetScalar(*(prhs.offset(3))) as f32;
            let g = mxGetScalar(*(prhs.offset(4))) as f32;
            let b = mxGetScalar(*(prhs.offset(5))) as f32;
            let a = mxGetScalar(*(prhs.offset(6))) as f32;

            strobe::clear_framebuffer(scr, fbo_ptr, r, g, b, a);
        },
        "draw_onto_framebuffer" => unsafe {
            if nrhs != 4 {
                genMatlabErrorMsg!(
                    "Strobe:draw_onto_framebuffer",
                    "Please provide the screen, onto whose framebuffer you would like to draw
                    a given shape. Also, please provide the associated framebuffer and the 
                    shape that should be drawn."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:draw_onto_framebuffer",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))
                && is_it_u64_w_type!(*(prhs.offset(2)))
                && is_it_u64_w_type!(*(prhs.offset(3))))
            {
                genMatlabErrorMsg!(
                    "Strobe:draw_onto_framebuffer",
                    "Please make sure that the first argument is a valid screen,
                    that the second argument is a valid shape, and that the third
                    argument is a valid framebuffer."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let p = *(prhs.offset(2)) as *const mxArray;
            let shape_addr = mxGetData(p) as *const u64;
            if *(shape_addr.offset(1)) != StrobeShape {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the second argument is a valid shape."
                );
            }
            let shape = std::mem::transmute::<u64, *mut strobe::ShapeEnum<'a>>(*shape_addr);

            let p = *(prhs.offset(3)) as *const mxArray;
            let fbo_ptr_addr = mxGetData(p) as *const u64;
            if *(fbo_ptr_addr.offset(1)) != StrobeFBO {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the third argument is a valid framebuffer."
                );
            }
            let fbo_ptr = std::mem::transmute::<u64, *mut glium::framebuffer::SimpleFrameBuffer>(
                *fbo_ptr_addr,
            );

            strobe::draw_onto_framebuffer(scr, shape, fbo_ptr);
        },

        // image
        "load_image" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:load_image",
                    "Please provide the filename of the image that should be loaded."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:load_image",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            // type check is done in macro
            let filename = get_chars_as_vec_i8!(prhs, 1, "load_image");

            let image = strobe::load_image((&filename).as_ptr()) as u64;

            let image_addr =
                mxCreateNumericMatrix(1, 2, mxClassID_mxUINT64_CLASS, mxComplexity_mxREAL);
            let matlab_addr = mxGetData(image_addr) as *mut u64;
            *matlab_addr = image;
            let strobe_type_addr = matlab_addr.offset(1);
            *strobe_type_addr = StrobeImage;
            let p = plhs.offset(0);
            *p = image_addr as *mut mxArray;
        },
        "image_mode" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:image_mode",
                    "Please provide the screen, whose image drawing mode should be
                    changed, as well as the new image drawing mode."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:image_mode",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:image_mode",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            // type check is done in macro
            let mode = get_chars_as_vec_i8!(prhs, 2, "image_mode");

            strobe::image_mode(scr, (&mode).as_ptr());
        },
        "no_tint" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:no_tint",
                    "Please provide the screen, where all image tinting should
                    be disabled."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:no_tint",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:no_tint",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::no_tint(scr);
        },
        "save" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:save",
                    "Please provide the screen, whose current display should be saved
                    to an image. Please also provide the filename where the image should
                    be saved. Note that currently, Strobe can only save PNG images, so
                    be sure that the file extension in the file name is \".png\"."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:save",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:save",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            // type check is done in macro
            let filename = get_chars_as_vec_i8!(prhs, 2, "save");

            strobe::save(scr, (&filename).as_ptr());
        },

        // input
        "key_press" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:key_press",
                    "Please provide the screen, whose keyboard input queue should be checked
                    for the given keypress. Please also provide the specific keyboard button
                    that should be checked for (It is easiest to use the convenient Key class)."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:key_press",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:key_press",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_u32_scalar!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:key_press",
                    "Please make sure that the second argument is a single valid key button.
                    It is easiest to use the convenient Key class. Please see the Strobe
                    documentation for details."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let button = mxGetScalar(*(prhs.offset(2))) as u32;

            let is_pressed = strobe::key_press(scr, button);

            let p = plhs.offset(0);
            *p = mxCreateDoubleScalar(is_pressed as f64);
        },
        "space_wait" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:space_wait",
                    "Please provide the screen that will receive keyboard input and halt
                    all execution of the program until the space bar is pressed."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:space_wait",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:space_wait",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::space_wait(scr);
        },
        "mouse_press" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:mouse_press",
                    "Please provide the screen, whose mouse input queue should be checked
                    for the given button press. Please also provide the specific mouse button
                    that should be checked for (It is easiest to use the convenient MouseButton class.)."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:mouse_press",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:mouse_press",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_u32_scalar!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:mouse_press",
                    "Please make sure that the second argument is a single valid mouse button.
                    It is easiest to use the convenient MouseButton class. Please see the Strobe
                    documentation for details."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let button = mxGetScalar(*(prhs.offset(2))) as u32;

            let is_pressed = strobe::mouse_press(scr, button);

            let p = plhs.offset(0);
            *p = mxCreateDoubleScalar(is_pressed as f64);
        },
        "mouse_release" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:mouse_release",
                    "Please provide the screen, whose mouse input queue should be checked
                    for the given keypress. Please also provide the specific mouse button
                    that should be checked for."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:mouse_release",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:mouse_release",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:mouse_release",
                    "Please make sure that the second argument is a single valid mouse button.
                    It is easiest to use the convenient MouseButton class. Please see the Strobe
                    documentation for details."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let button = mxGetScalar(*(prhs.offset(2))) as u32;

            let is_pressed = strobe::mouse_release(scr, button);

            let p = plhs.offset(0);
            *p = mxCreateDoubleScalar(is_pressed as f64);
        },
        "mouse_x" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:mouse_x",
                    "Please provide the screen that should be used for computing
                    the current relative X-coordinate of the mouse."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:mouse_x",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:mouse_x",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let mx = strobe::mouse_x(scr);

            let p = plhs.offset(0);
            *p = mxCreateDoubleScalar(mx as f64);
        },
        "mouse_y" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:mouse_y",
                    "Please provide the screen that should be used for computing
                    the current relative Y-coordinate of the mouse."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:mouse_y",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:mouse_y",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let my = strobe::mouse_y(scr);

            let p = plhs.offset(0);
            *p = mxCreateDoubleScalar(my as f64);
        },

        // mould
        "mould" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:mould",
                    "Please provide the shape and shader that should be merged into a
                    Mould."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:mould",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1))) && is_it_u64_w_type!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:mould",
                    "Please make sure that the first argument is a valid shape and
                    that the second argument is a valid shader."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let shape_addr = mxGetData(p) as *const u64;
            if *(shape_addr.offset(1)) != StrobeShape {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid shape."
                );
            }
            let shape = std::mem::transmute::<u64, *mut strobe::ShapeEnum<'a>>(*shape_addr);

            let p = *(prhs.offset(2)) as *const mxArray;
            let shader_ptr = mxGetData(p) as *const u64;
            if *(shader_ptr.offset(1)) != StrobeShader {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the second argument is a valid shader."
                );
            }
            let shader = std::mem::transmute::<u64, *mut p5::shaders::ShaderInfo<'a>>(*shader_ptr);

            let mould = strobe::mould(shape, shader) as u64;

            let mouldAddr =
                mxCreateNumericMatrix(1, 2, mxClassID_mxUINT64_CLASS, mxComplexity_mxREAL);
            let matlab_addr = mxGetData(mouldAddr) as *mut u64;
            *matlab_addr = mould;
            let strobe_type_addr = matlab_addr.offset(1);
            *strobe_type_addr = StrobeMould;
            let p = plhs.offset(0);
            *p = mouldAddr as *mut mxArray;
        },
        "draw_mould" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:draw_mould",
                    "Please provide the Mould that should be drawn and which screen
                    it should be drawn to."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:draw_mould",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1))) && is_it_u64_w_type!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:draw_mould",
                    "Please make sure that the first argument is a valid screen
                    and that the second argument is a valid Mould."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let p = *(prhs.offset(2)) as *const mxArray;
            let mould_addr = mxGetData(p) as *const u64;
            if *(mould_addr.offset(1)) != StrobeMould {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the second argument is a valid Mould."
                );
            }
            let mould = std::mem::transmute::<u64, *mut strobe::MouldExtern<'a>>(*mould_addr);

            strobe::draw_mould(scr, mould);
        },

        // rendering
        "blend_mode" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:blend_mode",
                    "Please provide the screen, whose blend mode should be changed.
                    Please also provide the specific blend mode that you would like."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:blend_mode",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:blend_mode",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            // type check is done in macro
            let mode = get_chars_as_vec_i8!(prhs, 2, "blend_mode");

            strobe::blend_mode(scr, (&mode).as_ptr());
        },

        // screen
        "screen" => unsafe {
            if nrhs != 6 {
                genMatlabErrorMsg!(
                    "Strobe:screen",
                    "Please provide the width and height for the new screen, as well as the following parameters:\n\nFullscreen = true/false,\nPreserve Aspect Ratio = true/false,\nHeadless = true/false."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:screen",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of the equals sign."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(1)))
                && is_it_floating_scalar!(*(prhs.offset(2))))
            {
                genMatlabErrorMsg!(
                    "Strobe:screen",
                    "Please make sure that the first and second arguments
                    (width and height of requsted screen) are positive valued
                    single numbers that are greater than zero."
                );
            }

            if !(mxIsLogicalScalar(*(prhs.offset(3)))
                && mxIsLogicalScalar(*(prhs.offset(4)))
                && mxIsLogicalScalar(*(prhs.offset(5))))
            {
                genMatlabErrorMsg!(
                    "Strobe:screen",
                    "Please make sure that the third, fourth, and fifth arguments
                    are all logical values (i.e., true or false)."
                );
            }

            let wid = mxGetScalar(*(prhs.offset(1))) as i32;
            let hei = mxGetScalar(*(prhs.offset(2))) as i32;

            if wid <= 0 || hei <= 0 {
                genMatlabErrorMsg!(
                    "Strobe:screen",
                    "Please make sure that the first and second arguments
                    (width and height of requsted screen) are positive valued
                    single numbers that are greater than zero."
                );
            }

            let full_scr_ptr = mxGetLogicals(*(prhs.offset(3)));
            let pres_asp_ratio_ptr = mxGetLogicals(*(prhs.offset(4)));
            let headless_ptr = mxGetLogicals(*(prhs.offset(5)));

            let scr = strobe::screen(
                wid as u32,
                hei as u32,
                *full_scr_ptr,
                *pres_asp_ratio_ptr,
                *headless_ptr,
            ) as u64;

            let windowAddr =
                mxCreateNumericMatrix(1, 2, mxClassID_mxUINT64_CLASS, mxComplexity_mxREAL);
            let matlab_addr = mxGetData(windowAddr) as *mut u64;
            *matlab_addr = scr;
            let strobe_type_addr = matlab_addr.offset(1);
            *strobe_type_addr = StrobeScreen;
            let p = plhs.offset(0);
            *p = windowAddr as *mut mxArray;
        },
        "reveal" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:reveal",
                    "Please provide the screen, whose display should be updated
                    to the visual result of all of the most recent drawing commands.
                    This is known as \"flipping\" or \"swapping\" in other graphics packages."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:reveal",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:reveal",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::reveal(scr);
        },
        "end_drawing" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:end_drawing",
                    "Please provide the screen that should be closed, ending its drawing
                    session and clearing all of its resources from memory. Any subsequent
                    attempts to draw to this screen will result in an error."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:end_drawing",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:end_drawing",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::end_drawing(scr);
        },

        // shaders
        "load_frag_shader" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:load_frag_shader",
                    "Please provide the screen, whose rendering context
                    will be used to load the fragment shader. Please
                    also provide the filename of the fragment shader."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:load_frag_shader",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:load_frag_shader",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            // type check is done in macro
            let filename = get_chars_as_vec_i8!(prhs, 2, "load_frag_shader");

            let frag_shader = strobe::load_frag_shader(scr, (&filename).as_ptr()) as u64;

            let shaderAddr =
                mxCreateNumericMatrix(1, 2, mxClassID_mxUINT64_CLASS, mxComplexity_mxREAL);
            let matlab_addr = mxGetData(shaderAddr) as *mut u64;
            *matlab_addr = frag_shader;
            let strobe_type_addr = matlab_addr.offset(1);
            *strobe_type_addr = StrobeShader;
            let p = plhs.offset(0);
            *p = shaderAddr as *mut mxArray;
        },
        "shader" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:shader",
                    "Please provide the name of the shader that will apply
                    to future rendering operations for the given screen.
                    Please also provide that screen."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:shader",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1))) && is_it_u64_w_type!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:shader",
                    "Please make sure that the first argument is a valid screen
                    and that the second argument is a valid shader."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let p = *(prhs.offset(2)) as *const mxArray;
            let shader_ptr_addr = mxGetData(p) as *const u64;
            if *(shader_ptr_addr.offset(1)) != StrobeShader {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the second argument is a valid shader."
                );
            }
            let shader_ptr =
                std::mem::transmute::<u64, *mut p5::shaders::ShaderInfo<'a>>(*shader_ptr_addr);

            strobe::shader(scr, shader_ptr);
        },
        "reset_shader" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:reset_shader",
                    "Please provide the screen that should be reset to its default
                    rendering shader."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:reset_shader",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:reset_shader",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::reset_shader(scr);
        },

        // shapes
        "arc" => unsafe {
            if nrhs != 9 {
                genMatlabErrorMsg!("Strobe:arc",
                    "Please provide a screen for creating the desired arc.
                    Please also provide the following parameters:
                    xc = x-coordinate of center,
                    yc = y-coordinate of center,
                    zc = z-coordinate of center,
                    w = width of bounding box,
                    h = height of bounding box,
                    start = angle relative to positive half of x-axis (i.e., the right half) of screen
                    from which the arc should start, 
                    stop = angle relative to positive half of x-axis (i.e., the right half) of screen
                    at which the arc should stop."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:arc",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:arc",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))
                && is_it_floating_scalar!(*(prhs.offset(3)))
                && is_it_floating_scalar!(*(prhs.offset(4)))
                && is_it_floating_scalar!(*(prhs.offset(5)))
                && is_it_floating_scalar!(*(prhs.offset(6)))
                && is_it_floating_scalar!(*(prhs.offset(7)))
                && is_it_floating_scalar!(*(prhs.offset(8))))
            {
                genMatlabErrorMsg!(
                    "Strobe:arc",
                    "Please make sure that all arguments that specify the dimensions of the requested shape
                    are valid real numbers."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let xci = mxGetScalar(*(prhs.offset(2)));
            let yci = mxGetScalar(*(prhs.offset(3)));
            let zci = mxGetScalar(*(prhs.offset(4)));
            let wi = mxGetScalar(*(prhs.offset(5)));
            let hi = mxGetScalar(*(prhs.offset(6)));
            let starti = mxGetScalar(*(prhs.offset(7)));
            let stopi = mxGetScalar(*(prhs.offset(8)));

            let arc = strobe::arc(scr, xci, yci, zci, wi, hi, starti, stopi) as u64;

            let arcAddr =
                mxCreateNumericMatrix(1, 2, mxClassID_mxUINT64_CLASS, mxComplexity_mxREAL);
            let matlab_addr = mxGetData(arcAddr) as *mut u64;
            *matlab_addr = arc;
            let strobe_type_addr = matlab_addr.offset(1);
            *strobe_type_addr = StrobeShape;
            let p = plhs.offset(0);
            *p = arcAddr as *mut mxArray;
        },
        "ellipse" => unsafe {
            if nrhs != 7 {
                genMatlabErrorMsg!(
                    "Strobe:ellipse",
                    "Please provide a screen for creating the desired ellipse.
                    Please also provide the following parameters:
                    xc = x-coordinate of center,
                    yc = y-coordinate of center,
                    zc = z-coordinate of center,
                    w = width of bounding box,
                    h = height of bounding box."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:ellipse",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:ellipse",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))
                && is_it_floating_scalar!(*(prhs.offset(3)))
                && is_it_floating_scalar!(*(prhs.offset(4)))
                && is_it_floating_scalar!(*(prhs.offset(5)))
                && is_it_floating_scalar!(*(prhs.offset(6))))
            {
                genMatlabErrorMsg!(
                    "Strobe:ellipse",
                    "Please make sure that all arguments that specify the dimensions of the requested shape
                    are valid real numbers."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let xci = mxGetScalar(*(prhs.offset(2)));
            let yci = mxGetScalar(*(prhs.offset(3)));
            let zci = mxGetScalar(*(prhs.offset(4)));
            let wi = mxGetScalar(*(prhs.offset(5)));
            let hi = mxGetScalar(*(prhs.offset(6)));

            let ellipse = strobe::ellipse(scr, xci, yci, zci, wi, hi) as u64;

            let ellipseAddr =
                mxCreateNumericMatrix(1, 2, mxClassID_mxUINT64_CLASS, mxComplexity_mxREAL);
            let matlab_addr = mxGetData(ellipseAddr) as *mut u64;
            *matlab_addr = ellipse;
            let strobe_type_addr = matlab_addr.offset(1);
            *strobe_type_addr = StrobeShape;
            let p = plhs.offset(0);
            *p = ellipseAddr as *mut mxArray;
        },
        "line" => unsafe {
            if nrhs != 8 {
                genMatlabErrorMsg!(
                    "Strobe:line",
                    "Please provide a screen for creating the desired line.
                    Please also provide the following parameters:
                    x1 = x-coordinate of one endpoint of line,
                    y1 = y-coordinate of one endpoint of line,
                    z1 = z-coordinate of one endpoint of line,
                    x2 = x-coordinate of other endpoint of line,
                    y2 = y-coordinate of other endpoint of line,
                    z2 = z-coordinate of other endpoint of line"
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:line",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:line",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))
                && is_it_floating_scalar!(*(prhs.offset(3)))
                && is_it_floating_scalar!(*(prhs.offset(4)))
                && is_it_floating_scalar!(*(prhs.offset(5)))
                && is_it_floating_scalar!(*(prhs.offset(6)))
                && is_it_floating_scalar!(*(prhs.offset(7))))
            {
                genMatlabErrorMsg!(
                    "Strobe:line",
                    "Please make sure that all arguments that specify the dimensions of the requested shape
                    are valid real numbers."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let x1i = mxGetScalar(*(prhs.offset(2)));
            let y1i = mxGetScalar(*(prhs.offset(3)));
            let z1i = mxGetScalar(*(prhs.offset(4)));
            let x2i = mxGetScalar(*(prhs.offset(5)));
            let y2i = mxGetScalar(*(prhs.offset(6)));
            let z2i = mxGetScalar(*(prhs.offset(7)));

            let line = strobe::line(scr, x1i, y1i, z1i, x2i, y2i, z2i) as u64;

            let lineAddr =
                mxCreateNumericMatrix(1, 2, mxClassID_mxUINT64_CLASS, mxComplexity_mxREAL);
            let matlab_addr = mxGetData(lineAddr) as *mut u64;
            *matlab_addr = line;
            let strobe_type_addr = matlab_addr.offset(1);
            *strobe_type_addr = StrobeShape;
            let p = plhs.offset(0);
            *p = lineAddr as *mut mxArray;
        },
        "point" => unsafe {
            if nrhs != 5 {
                genMatlabErrorMsg!(
                    "Strobe:point",
                    "Please provide a screen for creating the desired point.
                    Please also provide the following parameters:
                    x = x-coordinate of point,
                    y = y-coordinate of point,
                    z = z-coordinate of point"
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:point",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:point",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))
                && is_it_floating_scalar!(*(prhs.offset(3)))
                && is_it_floating_scalar!(*(prhs.offset(4))))
            {
                genMatlabErrorMsg!(
                    "Strobe:point",
                    "Please make sure that all arguments that specify the dimensions of the requested shape
                    are valid real numbers."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let xi = mxGetScalar(*(prhs.offset(2)));
            let yi = mxGetScalar(*(prhs.offset(3)));
            let zi = mxGetScalar(*(prhs.offset(4)));

            let point = strobe::point(scr, xi, yi, zi) as u64;

            let pointAddr =
                mxCreateNumericMatrix(1, 2, mxClassID_mxUINT64_CLASS, mxComplexity_mxREAL);
            let matlab_addr = mxGetData(pointAddr) as *mut u64;
            *matlab_addr = point;
            let strobe_type_addr = matlab_addr.offset(1);
            *strobe_type_addr = StrobeShape;
            let p = plhs.offset(0);
            *p = pointAddr as *mut mxArray;
        },
        "quad" => unsafe {
            if nrhs != 14 {
                genMatlabErrorMsg!(
                    "Strobe:quad",
                    "Please provide a screen for creating the desired quadrilateral.
                    Please also provide the following parameters:
                    x1 = x-coordinate of top left vertex,
                    y1 = y-coordinate of top left vertex,
                    z1 = z-coordinate of top left vertex,
                    x2 = x-coordinate of top right vertex,
                    y2 = y-coordinate of top right vertex,
                    z2 = z-coordinate of top right vertex,
                    x3 = x-coordinate of bottom right vertex,
                    y3 = y-coordinate of bottom right vertex,
                    z3 = z-coordinate of bottom right vertex,
                    x4 = x-coordinate of bottom left vertex,
                    y4 = y-coordinate of bottom left vertex,
                    z4 = z-coordinate of bottom left vertex."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:quad",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:quad",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))
                && is_it_floating_scalar!(*(prhs.offset(3)))
                && is_it_floating_scalar!(*(prhs.offset(4)))
                && is_it_floating_scalar!(*(prhs.offset(5)))
                && is_it_floating_scalar!(*(prhs.offset(6)))
                && is_it_floating_scalar!(*(prhs.offset(7)))
                && is_it_floating_scalar!(*(prhs.offset(8)))
                && is_it_floating_scalar!(*(prhs.offset(9)))
                && is_it_floating_scalar!(*(prhs.offset(10)))
                && is_it_floating_scalar!(*(prhs.offset(11)))
                && is_it_floating_scalar!(*(prhs.offset(12)))
                && is_it_floating_scalar!(*(prhs.offset(13))))
            {
                genMatlabErrorMsg!(
                    "Strobe:quad",
                    "Please make sure that all arguments that specify the dimensions of the requested shape
                    are valid real numbers."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let x1i = mxGetScalar(*(prhs.offset(2)));
            let y1i = mxGetScalar(*(prhs.offset(3)));
            let z1i = mxGetScalar(*(prhs.offset(4)));
            let x2i = mxGetScalar(*(prhs.offset(5)));
            let y2i = mxGetScalar(*(prhs.offset(6)));
            let z2i = mxGetScalar(*(prhs.offset(7)));
            let x3i = mxGetScalar(*(prhs.offset(8)));
            let y3i = mxGetScalar(*(prhs.offset(9)));
            let z3i = mxGetScalar(*(prhs.offset(10)));
            let x4i = mxGetScalar(*(prhs.offset(11)));
            let y4i = mxGetScalar(*(prhs.offset(12)));
            let z4i = mxGetScalar(*(prhs.offset(13)));

            let quad = strobe::quad(
                scr, x1i, y1i, z1i, x2i, y2i, z2i, x3i, y3i, z3i, x4i, y4i, z4i,
            ) as u64;

            let quadAddr =
                mxCreateNumericMatrix(1, 2, mxClassID_mxUINT64_CLASS, mxComplexity_mxREAL);
            let matlab_addr = mxGetData(quadAddr) as *mut u64;
            *matlab_addr = quad;
            let strobe_type_addr = matlab_addr.offset(1);
            *strobe_type_addr = StrobeShape;
            let p = plhs.offset(0);
            *p = quadAddr as *mut mxArray;
        },
        "triangle" => unsafe {
            if nrhs != 11 {
                genMatlabErrorMsg!(
                    "Strobe:triangle",
                    "Please provide a screen for creating the desired triangle.
                    Please also provide the following parameters:
                    x1 = x-coordinate of top vertex,
                    y1 = y-coordinate of top vertex,
                    z1 = z-coordinate of top vertex,
                    x2 = x-coordinate of bottom right vertex,
                    y2 = y-coordinate of bottom right vertex,
                    z2 = z-coordinate of bottom right vertex,
                    x3 = x-coordinate of bottom left vertex,
                    y3 = y-coordinate of bottom left vertex,
                    z3 = z-coordinate of bottom left vertex"
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:triangle",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:triangle",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))
                && is_it_floating_scalar!(*(prhs.offset(3)))
                && is_it_floating_scalar!(*(prhs.offset(4)))
                && is_it_floating_scalar!(*(prhs.offset(5)))
                && is_it_floating_scalar!(*(prhs.offset(6)))
                && is_it_floating_scalar!(*(prhs.offset(7)))
                && is_it_floating_scalar!(*(prhs.offset(8)))
                && is_it_floating_scalar!(*(prhs.offset(9)))
                && is_it_floating_scalar!(*(prhs.offset(10))))
            {
                genMatlabErrorMsg!(
                    "Strobe:triangle",
                    "Please make sure that all arguments that specify the dimensions of the requested shape
                    are valid real numbers."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let x1i = mxGetScalar(*(prhs.offset(2)));
            let y1i = mxGetScalar(*(prhs.offset(3)));
            let z1i = mxGetScalar(*(prhs.offset(4)));
            let x2i = mxGetScalar(*(prhs.offset(5)));
            let y2i = mxGetScalar(*(prhs.offset(6)));
            let z2i = mxGetScalar(*(prhs.offset(7)));
            let x3i = mxGetScalar(*(prhs.offset(8)));
            let y3i = mxGetScalar(*(prhs.offset(9)));
            let z3i = mxGetScalar(*(prhs.offset(10)));

            let triangle =
                strobe::triangle(scr, x1i, y1i, z1i, x2i, y2i, z2i, x3i, y3i, z3i) as u64;

            let triAddr =
                mxCreateNumericMatrix(1, 2, mxClassID_mxUINT64_CLASS, mxComplexity_mxREAL);
            let matlab_addr = mxGetData(triAddr) as *mut u64;
            *matlab_addr = triangle;
            let strobe_type_addr = matlab_addr.offset(1);
            *strobe_type_addr = StrobeShape;
            let p = plhs.offset(0);
            *p = triAddr as *mut mxArray;
        },
        "rect" => unsafe {
            if nrhs != 7 {
                genMatlabErrorMsg!(
                    "Strobe:rect",
                    "Please provide a screen for creating the desired rectangle.
                    Please also provide the following parameters:
                    xtopleft = x-coordinate of top left vertex,
                    ytopleft = y-coordinate of top left vertex,
                    ztoplift = z-coordinate of top left vertex,
                    width = width of rectangle,
                    height = height of rectangle"
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:rect",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:rect",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))
                && is_it_floating_scalar!(*(prhs.offset(3)))
                && is_it_floating_scalar!(*(prhs.offset(4)))
                && is_it_floating_scalar!(*(prhs.offset(5)))
                && is_it_floating_scalar!(*(prhs.offset(6))))
            {
                genMatlabErrorMsg!(
                    "Strobe:rect",
                    "Please make sure that all arguments that specify the dimensions of the requested shape
                    are valid real numbers."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let xtoplefti = mxGetScalar(*(prhs.offset(2)));
            let ytoplefti = mxGetScalar(*(prhs.offset(3)));
            let ztoplefti = mxGetScalar(*(prhs.offset(4)));
            let widthi = mxGetScalar(*(prhs.offset(5)));
            let heighti = mxGetScalar(*(prhs.offset(6)));

            let rect = strobe::rect(scr, xtoplefti, ytoplefti, ztoplefti, widthi, heighti) as u64;

            let rectAddr =
                mxCreateNumericMatrix(1, 2, mxClassID_mxUINT64_CLASS, mxComplexity_mxREAL);
            let matlab_addr = mxGetData(rectAddr) as *mut u64;
            *matlab_addr = rect;
            let strobe_type_addr = matlab_addr.offset(1);
            *strobe_type_addr = StrobeShape;
            let p = plhs.offset(0);
            *p = rectAddr as *mut mxArray;
        },
        "cube" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:cube",
                    "Please provide a screen for creating the desired cube.
                    Please also provide the following parameter:
                    s = size of cube"
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:cube",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:cube",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:cube",
                    "Please make sure that all arguments that specify the dimensions of the requested shape
                    are valid real numbers."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let s = mxGetScalar(*(prhs.offset(2)));

            let cube = strobe::cube(scr, s) as u64;

            let cubeAddr =
                mxCreateNumericMatrix(1, 2, mxClassID_mxUINT64_CLASS, mxComplexity_mxREAL);
            let matlab_addr = mxGetData(cubeAddr) as *mut u64;
            *matlab_addr = cube;
            let strobe_type_addr = matlab_addr.offset(1);
            *strobe_type_addr = StrobeShape;
            let p = plhs.offset(0);
            *p = cubeAddr as *mut mxArray;
        },
        "draw" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:draw",
                    "Please provide the screen, where you want the shape to be drawn.
                    Please also provide the shape."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:draw",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1))) && is_it_u64_w_type!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:draw",
                    "Please make sure that the first argument is a valid screen
                    and that the second argument is a valid shape."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let p = *(prhs.offset(2)) as *const mxArray;
            let shape_matlab_addr = mxGetData(p) as *const u64;
            if *(shape_matlab_addr.offset(1)) != StrobeShape {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the second argument is a valid shape."
                );
            }
            let shape = std::mem::transmute::<u64, *mut strobe::ShapeEnum<'a>>(*shape_matlab_addr);

            strobe::draw(scr, shape);
        },
        "stroke_weight" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:stroke_weight",
                    "Please provide the screen, where you want to change the weight
                    of the lines used for drawing the stroke. Please also provide
                    the new weight."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:stroke_weight",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:stroke_weight",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:stroke_weight",
                    "Please make sure that the second argument is a positive number (i.e., >=0)."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let new_weight = mxGetScalar(*(prhs.offset(2))) as f32;

            if new_weight < 0f32 {
                genMatlabErrorMsg!(
                    "Strobe:stroke_weight",
                    "Please make sure that the second argument is a positive number (i.e., >=0)."
                );
            }

            strobe::stroke_weight(scr, new_weight);
        },
        "ellipse_mode" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:ellipse_mode",
                    "Please provide the screen, where you want to change the drawing
                    mode for ellipses. Please also provide the desired mode."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:ellipse_mode",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:ellipse_mode",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            // type check is done in macro
            let mode = get_chars_as_vec_i8!(prhs, 2, "ellipse_mode");

            strobe::ellipse_mode(scr, (&mode).as_ptr());
        },
        "rect_mode" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:rect_mode",
                    "Please provide the screen, where you want to change the drawing
                    mode for rectangles. Please also provide the desired mode."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:rect_mode",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:rect_mode",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            // type check is done in macro
            let mode = get_chars_as_vec_i8!(prhs, 2, "rect_mode");

            strobe::rect_mode(scr, (&mode).as_ptr());
        },
        "shape_mode" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:shape_mode",
                    "Please provide the screen, where you want to change the drawing
                    mode for general shapes. Please also provide the desired mode."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:shape_mode",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:shape_mode",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            // type check is done in macro
            let mode = get_chars_as_vec_i8!(prhs, 2, "shape_mode");

            strobe::shape_mode(scr, (&mode).as_ptr());
        },

        // textures
        "texture" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:texture",
                    "Please provide the screen, whose rendering context will contain
                    the desired texure. Please also provide the image for
                    generating that texture."
                );
            }

            if nlhs > 1 {
                genMatlabErrorMsg!(
                    "Strobe:texture",
                    "Please have at most one output argument for this function.
                    Remove any extra variable names from the left-hand side of
                    the equals sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1))) && is_it_u64_w_type!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:texture",
                    "Please make sure that the first argument is a valid screen
                    and that the second argument is a valid texture."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let p = *(prhs.offset(2)) as *const mxArray;
            let image_addr = mxGetData(p) as *const u64;
            if *(image_addr.offset(1)) != StrobeImage {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the second argument is a valid Strobe image."
                );
            }
            let image = std::mem::transmute::<u64, *mut image_ext::RgbaImage>(*image_addr);

            let texture = strobe::texture(scr, image) as u64;

            let texAddr =
                mxCreateNumericMatrix(1, 2, mxClassID_mxUINT64_CLASS, mxComplexity_mxREAL);
            let matlab_addr = mxGetData(texAddr) as *mut u64;
            *matlab_addr = texture;
            let strobe_type_addr = matlab_addr.offset(1);
            *strobe_type_addr = StrobeTexture;
            let p = plhs.offset(0);
            *p = texAddr as *mut mxArray;
        },
        "attach_texture" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:attach_texture",
                    "Please provide the shape to which the texture should be
                    attached (for the moment, only rectangles are supported).
                    Please also provide the texture."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:attach_texture",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1))) && is_it_u64_w_type!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:attach_texture",
                    "Please make sure that the first argument is a valid shape (at the moment, only
                    rectangles can hold textures) and that the second argument is a valid texture."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let shape_addr = mxGetData(p) as *const u64;
            if *(shape_addr.offset(1)) != StrobeShape {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the second argument is a valid shape (at the moment, only
                        rectangles can hold textures)."
                );
            }
            let shape = std::mem::transmute::<u64, *mut strobe::ShapeEnum<'a>>(*shape_addr);

            let p = *(prhs.offset(2)) as *const mxArray;
            let tex_addr = mxGetData(p) as *const u64;
            if *(tex_addr.offset(1)) != StrobeTexture {
                genMatlabErrorMsg!(
                    "Strobe:background",
                    "Please make sure that the second argument is a valid texture."
                );
            }
            let tex = std::mem::transmute::<u64, *mut glium::texture::Texture2d>(*tex_addr);

            strobe::attach_texture(shape, tex);
        },

        // transforms
        "apply_matrix" => unsafe {
            if nrhs != 18 {
                genMatlabErrorMsg!(
                    "Strobe:apply_matrix",
                    "Please provide the screen, whose rendering context will be
                    transformed by the given matrix. Please be sure to provide all
                    16 elements of the 4x4 transformation matrix (see Strobe documentation
                    for more details)."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:apply_matrix",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:apply_matrix",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))
                && is_it_floating_scalar!(*(prhs.offset(3)))
                && is_it_floating_scalar!(*(prhs.offset(4)))
                && is_it_floating_scalar!(*(prhs.offset(5)))
                && is_it_floating_scalar!(*(prhs.offset(6)))
                && is_it_floating_scalar!(*(prhs.offset(7)))
                && is_it_floating_scalar!(*(prhs.offset(8)))
                && is_it_floating_scalar!(*(prhs.offset(9)))
                && is_it_floating_scalar!(*(prhs.offset(10)))
                && is_it_floating_scalar!(*(prhs.offset(11)))
                && is_it_floating_scalar!(*(prhs.offset(12)))
                && is_it_floating_scalar!(*(prhs.offset(13)))
                && is_it_floating_scalar!(*(prhs.offset(14)))
                && is_it_floating_scalar!(*(prhs.offset(15)))
                && is_it_floating_scalar!(*(prhs.offset(16)))
                && is_it_floating_scalar!(*(prhs.offset(17))))
            {
                genMatlabErrorMsg!(
                    "Strobe:apply_matrix",
                    "Please make sure that all arguments that specify the elements of the transform matrix
                    are valid real numbers."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:apply_matrix",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let n00 = mxGetScalar(*(prhs.offset(2))) as f32;
            let n01 = mxGetScalar(*(prhs.offset(3))) as f32;
            let n02 = mxGetScalar(*(prhs.offset(4))) as f32;
            let n03 = mxGetScalar(*(prhs.offset(5))) as f32;
            let n10 = mxGetScalar(*(prhs.offset(6))) as f32;
            let n11 = mxGetScalar(*(prhs.offset(7))) as f32;
            let n12 = mxGetScalar(*(prhs.offset(8))) as f32;
            let n13 = mxGetScalar(*(prhs.offset(9))) as f32;
            let n20 = mxGetScalar(*(prhs.offset(10))) as f32;
            let n21 = mxGetScalar(*(prhs.offset(11))) as f32;
            let n22 = mxGetScalar(*(prhs.offset(12))) as f32;
            let n23 = mxGetScalar(*(prhs.offset(13))) as f32;
            let n30 = mxGetScalar(*(prhs.offset(14))) as f32;
            let n31 = mxGetScalar(*(prhs.offset(15))) as f32;
            let n32 = mxGetScalar(*(prhs.offset(16))) as f32;
            let n33 = mxGetScalar(*(prhs.offset(17))) as f32;

            strobe::apply_matrix(
                scr, n00, n01, n02, n03, n10, n11, n12, n13, n20, n21, n22, n23, n30, n31, n32, n33,
            );
        },
        "pop_matrix" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:pop_matrix",
                    "Please provide the screen, to which you will apply the most
                    recently saved transformation matrix (stored in the transformation matrix stack;
                    see Strobe documentation for more details)."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:pop_matrix",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:pop_matrix",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:pop_matrix",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::pop_matrix(scr);
        },
        "push_matrix" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:push_matrix",
                    "Please provide the screen, for which you will save the
                    currently applied transformation matrix (stored in the transformation matrix stack;
                    see Strobe documentation for more details)."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:push_matrix",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:push_matrix",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:push_matrix",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::push_matrix(scr);
        },
        "reset_matrix" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:reset_matrix",
                    "Please provide the screen, whose associated transformation will be reset
                    to the default (i.e., the 4x4 identity matrix; see Strobe documentation
                    for more details)."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:reset_matrix",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:reset_matrix",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:reset_matrix",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::reset_matrix(scr);
        },
        "rotate" => unsafe {
            if nrhs != 6 {
                genMatlabErrorMsg!(
                    "Strobe:rotate",
                    "Please provide the screen, whose rendering context will be rotated
                    by a given angle about a given axis. Please make sure to also provide
                    the angle and the 3 components of the desired rotation axis."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:rotate",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:rotate",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))
                && is_it_floating_scalar!(*(prhs.offset(3)))
                && is_it_floating_scalar!(*(prhs.offset(4)))
                && is_it_floating_scalar!(*(prhs.offset(5))))
            {
                genMatlabErrorMsg!(
                    "Strobe:rotate",
                    "Please make sure that all arguments that specify the angle of rotation and the
                    rotation axis are real numbers."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:rotate",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let angle = mxGetScalar(*(prhs.offset(2))) as f32;
            let x = mxGetScalar(*(prhs.offset(3))) as f32;
            let y = mxGetScalar(*(prhs.offset(4))) as f32;
            let z = mxGetScalar(*(prhs.offset(5))) as f32;

            strobe::rotate(scr, angle, x, y, z);
        },
        "rotate_x" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:rotate_x",
                    "Please provide the screen, whose rendering context will be rotated
                    by a given angle about the X-axis. Please make sure to also provide
                    the angle."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:rotate_x",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:rotate_x",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:rotate_x",
                    "Please make sure that the rotation angle about the X-axis is a real number."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:rotate_x",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let angle = mxGetScalar(*(prhs.offset(2))) as f32;

            strobe::rotate_x(scr, angle);
        },
        "rotate_y" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:rotate_y",
                    "Please provide the screen, whose rendering context will be rotated
                    by a given angle about the Y-axis. Please make sure to also provide
                    the angle."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:rotate_y",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:rotate_y",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:rotate_y",
                    "Please make sure that the rotation angle about the Y-axis is a real number."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:rotate_y",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let angle = mxGetScalar(*(prhs.offset(2))) as f32;

            strobe::rotate_y(scr, angle);
        },
        "rotate_z" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:rotate_z",
                    "Please provide the screen, whose rendering context will be rotated
                    by a given angle about the Z-axis. Please make sure to also provide
                    the angle."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:rotate_z",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:rotate_z",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:rotate_z",
                    "Please make sure that the rotation angle about the Z-axis is a real number."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:rotate_z",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let angle = mxGetScalar(*(prhs.offset(2))) as f32;

            strobe::rotate_z(scr, angle);
        },
        "scale" => unsafe {
            if nrhs != 5 {
                genMatlabErrorMsg!(
                    "Strobe:scale",
                    "Please provide the screen, whose rendering context will be scaled
                    along the X-, Y-, and Z-axes. Please make sure to also provide
                    the 3 scaling components (X, Y, and Z)."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:scale",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:scale",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))
                && is_it_floating_scalar!(*(prhs.offset(3)))
                && is_it_floating_scalar!(*(prhs.offset(4))))
            {
                genMatlabErrorMsg!(
                    "Strobe:scale",
                    "Please make sure that the 3 scaling factors (X, Y, and Z) are real numbers."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:scale",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let x = mxGetScalar(*(prhs.offset(2))) as f32;
            let y = mxGetScalar(*(prhs.offset(3))) as f32;
            let z = mxGetScalar(*(prhs.offset(4))) as f32;

            strobe::scale(scr, x, y, z);
        },
        "shear_x" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:shear_x",
                    "Please provide the screen, whose rendering context will be sheared
                    by a given angle along the X-axis. Please make sure to also provide
                    the shearing angle."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:shear_x",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:shear_x",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:shear_x",
                    "Please make sure that the shearing angle is a real number."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:shear_x",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let angle = mxGetScalar(*(prhs.offset(2))) as f32;

            strobe::shear_x(scr, angle);
        },
        "shear_y" => unsafe {
            if nrhs != 3 {
                genMatlabErrorMsg!(
                    "Strobe:shear_y",
                    "Please provide the screen, whose rendering context will be sheared
                    by a given angle along the Y-axis. Please make sure to also provide
                    the shearing angle."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:shear_y",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:shear_y",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))) {
                genMatlabErrorMsg!(
                    "Strobe:shear_y",
                    "Please make sure that the shearing angle is a real number."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:shear_y",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let angle = mxGetScalar(*(prhs.offset(2))) as f32;

            strobe::shear_y(scr, angle);
        },
        "translate" => unsafe {
            if nrhs != 5 {
                genMatlabErrorMsg!(
                    "Strobe:translate",
                    "Please provide the screen, whose rendering context will be translated
                    by the given translation vector. Please make sure to also provide
                    the 3 components (X, Y, and Z) of the desired translation vector."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:translate",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:translate",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            if !(is_it_floating_scalar!(*(prhs.offset(2)))
                && is_it_floating_scalar!(*(prhs.offset(3)))
                && is_it_floating_scalar!(*(prhs.offset(4))))
            {
                genMatlabErrorMsg!(
                    "Strobe:translate",
                    "Please make sure that the 3 components (X, Y, and Z) of the translation
                    vector are real numbers."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:translate",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            let x = mxGetScalar(*(prhs.offset(2))) as f32;
            let y = mxGetScalar(*(prhs.offset(3))) as f32;
            let z = mxGetScalar(*(prhs.offset(4))) as f32;

            strobe::translate(scr, x, y, z);
        },
        "print_matrix" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:print_matrix",
                    "Please provide the screen, whose current transformation matrix
                    you would like to print out."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:print_matrix",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:print_matrix",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:print_matrix",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let scr = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::print_matrix(scr);
        },

        // uniforms
        "shader_set_i8" => unsafe {
            if nrhs != 4 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i8",
                    "Please provide the shader, whose uniform value you would like to change,
                    as well as the name of the uniform and its new value."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i8",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i8",
                    "Please make sure that the first argument is a valid shader."
                );
            }

            if !mxIsInt8(*(prhs.offset(3))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i8",
                    "Please make sure that the fourth argument is an 8-bit integer (int8)."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let shader_matlab_addr = mxGetData(p) as *const u64;
            if *(shader_matlab_addr.offset(1)) != StrobeShader {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i8",
                    "Please make sure that the first argument is a valid shader."
                );
            }
            let shader =
                std::mem::transmute::<u64, *mut p5::shaders::ShaderInfo<'a>>(*shader_matlab_addr);

            // type check is done in macro
            let uniform_name = get_chars_as_vec_i8!(prhs, 2, "shader_set_i8");

            let uniform_val = *mxGetInt8s_800(*(prhs.offset(3)));

            strobe::shader_set_i8(
                shader as *mut c_void,
                uniform_name[..].as_ptr(),
                uniform_val,
            );
        },
        "shader_set_u8" => unsafe {
            if nrhs != 4 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u8",
                    "Please provide the shader, whose uniform value you would like to change,
                    as well as the name of the uniform and its new value."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u8",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u8",
                    "Please make sure that the first argument is a valid shader."
                );
            }

            if !mxIsUint8(*(prhs.offset(3))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u8",
                    "Please make sure that the fourth argument is an unsigned 8-bit integer (uint8)."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let shader_matlab_addr = mxGetData(p) as *const u64;
            if *(shader_matlab_addr.offset(1)) != StrobeShader {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u8",
                    "Please make sure that the first argument is a valid shader."
                );
            }
            let shader =
                std::mem::transmute::<u64, *mut p5::shaders::ShaderInfo<'a>>(*shader_matlab_addr);

            // type check is done in macro
            let uniform_name = get_chars_as_vec_i8!(prhs, 2, "shader_set_u8");

            let uniform_val = *mxGetUint8s_800(*(prhs.offset(3)));

            strobe::shader_set_u8(
                shader as *mut c_void,
                uniform_name[..].as_ptr(),
                uniform_val,
            );
        },
        "shader_set_i16" => unsafe {
            if nrhs != 4 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i16",
                    "Please provide the shader, whose uniform value you would like to change,
                    as well as the name of the uniform and its new value."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i16",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i16",
                    "Please make sure that the first argument is a valid shader."
                );
            }

            if !mxIsInt16(*(prhs.offset(3))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i16",
                    "Please make sure that the fourth argument is a 16-bit integer (int16)."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let shader_matlab_addr = mxGetData(p) as *const u64;
            if *(shader_matlab_addr.offset(1)) != StrobeShader {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i16",
                    "Please make sure that the first argument is a valid shader."
                );
            }
            let shader =
                std::mem::transmute::<u64, *mut p5::shaders::ShaderInfo<'a>>(*shader_matlab_addr);

            // type check is done in macro
            let uniform_name = get_chars_as_vec_i8!(prhs, 2, "shader_set_i16");

            let uniform_val = *mxGetInt16s_800(*(prhs.offset(3)));

            strobe::shader_set_i16(
                shader as *mut c_void,
                uniform_name[..].as_ptr(),
                uniform_val,
            );
        },
        "shader_set_u16" => unsafe {
            if nrhs != 4 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u16",
                    "Please provide the shader, whose uniform value you would like to change,
                    as well as the name of the uniform and its new value."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u16",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u16",
                    "Please make sure that the first argument is a valid shader."
                );
            }

            if !mxIsUint16(*(prhs.offset(3))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u16",
                    "Please make sure that the fourth argument is an unsigned 16-bit integer (uint16)."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let shader_matlab_addr = mxGetData(p) as *const u64;
            if *(shader_matlab_addr.offset(1)) != StrobeShader {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u16",
                    "Please make sure that the first argument is a valid shader."
                );
            }
            let shader =
                std::mem::transmute::<u64, *mut p5::shaders::ShaderInfo<'a>>(*shader_matlab_addr);

            // type check is done in macro
            let uniform_name = get_chars_as_vec_i8!(prhs, 2, "shader_set_u16");

            let uniform_val = *mxGetUint16s_800(*(prhs.offset(3)));

            strobe::shader_set_u16(
                shader as *mut c_void,
                uniform_name[..].as_ptr(),
                uniform_val,
            );
        },
        "shader_set_i32" => unsafe {
            if nrhs != 4 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i32",
                    "Please provide the shader, whose uniform value you would like to change,
                    as well as the name of the uniform and its new value."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i32",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i32",
                    "Please make sure that the first argument is a valid shader."
                );
            }

            if !mxIsInt32(*(prhs.offset(3))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i32",
                    "Please make sure that the fourth argument is a 32-bit integer (int32)."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let shader_matlab_addr = mxGetData(p) as *const u64;
            if *(shader_matlab_addr.offset(1)) != StrobeShader {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i32",
                    "Please make sure that the first argument is a valid shader."
                );
            }
            let shader =
                std::mem::transmute::<u64, *mut p5::shaders::ShaderInfo<'a>>(*shader_matlab_addr);

            // type check is done in macro
            let uniform_name = get_chars_as_vec_i8!(prhs, 2, "shader_set_i32");

            let uniform_val = *mxGetInt32s_800(*(prhs.offset(3)));

            strobe::shader_set_i32(
                shader as *mut c_void,
                uniform_name[..].as_ptr(),
                uniform_val,
            );
        },
        "shader_set_u32" => unsafe {
            if nrhs != 4 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u32",
                    "Please provide the shader, whose uniform value you would like to change,
                    as well as the name of the uniform and its new value."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u32",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u32",
                    "Please make sure that the first argument is a valid shader."
                );
            }

            if !mxIsUint32(*(prhs.offset(3))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u32",
                    "Please make sure that the fourth argument is an unsigned 32-bit integer (uint32)."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let shader_matlab_addr = mxGetData(p) as *const u64;
            if *(shader_matlab_addr.offset(1)) != StrobeShader {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u32",
                    "Please make sure that the first argument is a valid shader."
                );
            }
            let shader =
                std::mem::transmute::<u64, *mut p5::shaders::ShaderInfo<'a>>(*shader_matlab_addr);

            // type check is done in macro
            let uniform_name = get_chars_as_vec_i8!(prhs, 2, "shader_set_u32");

            let uniform_val = *mxGetUint32s_800(*(prhs.offset(3)));

            strobe::shader_set_u32(
                shader as *mut c_void,
                uniform_name[..].as_ptr(),
                uniform_val,
            );
        },
        "shader_set_i64" => unsafe {
            if nrhs != 4 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i64",
                    "Please provide the shader, whose uniform value you would like to change,
                    as well as the name of the uniform and its new value."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i64",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i64",
                    "Please make sure that the first argument is a valid shader."
                );
            }

            if !mxIsInt64(*(prhs.offset(3))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i64",
                    "Please make sure that the fourth argument is a 64-bit integer (int64)."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let shader_matlab_addr = mxGetData(p) as *const u64;
            if *(shader_matlab_addr.offset(1)) != StrobeShader {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_i64",
                    "Please make sure that the first argument is a valid shader."
                );
            }
            let shader =
                std::mem::transmute::<u64, *mut p5::shaders::ShaderInfo<'a>>(*shader_matlab_addr);

            // type check is done in macro
            let uniform_name = get_chars_as_vec_i8!(prhs, 2, "shader_set_i64");

            let uniform_val = *mxGetInt64s_800(*(prhs.offset(3)));

            strobe::shader_set_i64(
                shader as *mut c_void,
                uniform_name[..].as_ptr(),
                uniform_val as i32, // FIXME
            );
        },
        "shader_set_u64" => unsafe {
            if nrhs != 4 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u64",
                    "Please provide the shader, whose uniform value you would like to change,
                    as well as the name of the uniform and its new value."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u64",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u64",
                    "Please make sure that the first argument is a valid shader."
                );
            }

            if !mxIsUint64(*(prhs.offset(3))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u64",
                    "Please make sure that the fourth argument is an unsigned 64-bit integer (uint64)."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let shader_matlab_addr = mxGetData(p) as *const u64;
            if *(shader_matlab_addr.offset(1)) != StrobeShader {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_u64",
                    "Please make sure that the first argument is a valid shader."
                );
            }
            let shader =
                std::mem::transmute::<u64, *mut p5::shaders::ShaderInfo<'a>>(*shader_matlab_addr);

            // type check is done in macro
            let uniform_name = get_chars_as_vec_i8!(prhs, 2, "shader_set_u64");

            let uniform_val = *mxGetUint64s_800(*(prhs.offset(3)));

            strobe::shader_set_u64(
                shader as *mut c_void,
                uniform_name[..].as_ptr(),
                uniform_val as u32, // FIXME
            );
        },
        "shader_set_f32" => unsafe {
            if nrhs != 4 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_f32",
                    "Please provide the shader, whose uniform value you would like to change,
                    as well as the name of the uniform and its new value."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_f32",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_f32",
                    "Please make sure that the first argument is a valid shader."
                );
            }

            if !mxIsSingle(*(prhs.offset(3))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_f32",
                    "Please make sure that the fourth argument is a single (32-bit floating point number)."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let shader_matlab_addr = mxGetData(p) as *const u64;
            if *(shader_matlab_addr.offset(1)) != StrobeShader {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_f32",
                    "Please make sure that the first argument is a valid shader."
                );
            }
            let shader =
                std::mem::transmute::<u64, *mut p5::shaders::ShaderInfo<'a>>(*shader_matlab_addr);

            // type check is done in macro
            let uniform_name = get_chars_as_vec_i8!(prhs, 2, "shader_set_f32");

            let uniform_val = *mxGetSingles_800(*(prhs.offset(3)));

            strobe::shader_set_f32(
                shader as *mut c_void,
                uniform_name[..].as_ptr(),
                uniform_val,
            );
        },
        "shader_set_f64" => unsafe {
            if nrhs != 4 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_f64",
                    "Please provide the shader, whose uniform value you would like to change,
                    as well as the name of the uniform and its new value."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_f64",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_f64",
                    "Please make sure that the first argument is a valid shader."
                );
            }

            if !mxIsDouble(*(prhs.offset(3))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_f64",
                    "Please make sure that the fourth argument is a double (64-bit floating point number)."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let shader_matlab_addr = mxGetData(p) as *const u64;
            if *(shader_matlab_addr.offset(1)) != StrobeShader {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_f64",
                    "Please make sure that the first argument is a valid shader."
                );
            }
            let shader =
                std::mem::transmute::<u64, *mut p5::shaders::ShaderInfo<'a>>(*shader_matlab_addr);

            // type check is done in macro
            let uniform_name = get_chars_as_vec_i8!(prhs, 2, "shader_set_f64");

            let uniform_val = *mxGetDoubles_800(*(prhs.offset(3)));

            strobe::shader_set_f64(
                shader as *mut c_void,
                uniform_name[..].as_ptr(),
                uniform_val,
            );
        },
        "shader_set_bool" => unsafe {
            if nrhs != 4 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_bool",
                    "Please provide the shader, whose uniform value you would like to change,
                    as well as the name of the uniform and its new value."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_bool",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_bool",
                    "Please make sure that the first argument is a valid shader."
                );
            }

            if !mxIsLogical(*(prhs.offset(3))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_bool",
                    "Please make sure that the fourth argument is a boolean (logical value -> true or false)."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let shader_matlab_addr = mxGetData(p) as *const u64;
            if *(shader_matlab_addr.offset(1)) != StrobeShader {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_bool",
                    "Please make sure that the first argument is a valid shader."
                );
            }
            let shader =
                std::mem::transmute::<u64, *mut p5::shaders::ShaderInfo<'a>>(*shader_matlab_addr);

            // type check is done in macro
            let uniform_name = get_chars_as_vec_i8!(prhs, 2, "shader_set_bool");

            let uniform_val = *mxGetLogicals(*(prhs.offset(3)));

            strobe::shader_set_bool(
                shader as *mut c_void,
                uniform_name[..].as_ptr(),
                uniform_val,
            );
        },
        "shader_set_tex" => unsafe {
            if nrhs != 4 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_tex",
                    "Please provide the shader, whose uniform value you would like to change,
                    as well as the name of the uniform and its new value."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_tex",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1))) && is_it_u64_w_type!(*(prhs.offset(3)))) {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_tex",
                    "Please make sure that the first argument is a valid shader and that the
                    third argument is a valid texture."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let shader_matlab_addr = mxGetData(p) as *const u64;
            if *(shader_matlab_addr.offset(1)) != StrobeShader {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_tex",
                    "Please make sure that the first argument is a valid shader."
                );
            }
            let shader =
                std::mem::transmute::<u64, *mut p5::shaders::ShaderInfo<'a>>(*shader_matlab_addr);

            // type check is done in macro
            let uniform_name = get_chars_as_vec_i8!(prhs, 2, "shader_set_tex");

            let p = *(prhs.offset(3)) as *const mxArray;
            let texture_matlab_addr = mxGetData(p) as *const u64;
            if *(texture_matlab_addr.offset(1)) != StrobeTexture {
                genMatlabErrorMsg!(
                    "Strobe:shader_set_tex",
                    "Please make sure that the thrid argument is a valid texture."
                );
            }
            let uniform_val =
                std::mem::transmute::<u64, *const glium::texture::Texture2d>(*texture_matlab_addr);

            strobe::shader_set_tex(shader, uniform_name[..].as_ptr(), uniform_val);
        },

        // drop
        "drop_screen" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:drop_screen",
                    "Please provide the screen that should be dropped."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:drop_screen",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:drop_screen",
                    "Please make sure that the first argument is a valid screen."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let screen_matlab_addr = mxGetData(p) as *const u64;
            if *(screen_matlab_addr.offset(1)) != StrobeScreen {
                genMatlabErrorMsg!(
                    "Strobe:drop_screen",
                    "Please make sure that the first argument is a valid screen."
                );
            }
            let screen = std::mem::transmute::<u64, *mut p5::Screen<'a>>(*screen_matlab_addr);

            strobe::drop_screen(screen);
        },
        "drop_shape" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:drop_shape",
                    "Please provide the shape that should be dropped."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:drop_shape",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:drop_shape",
                    "Please make sure that the first argument is a valid shape."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let shape_matlab_addr = mxGetData(p) as *const u64;
            if *(shape_matlab_addr.offset(1)) != StrobeShape {
                genMatlabErrorMsg!(
                    "Strobe:drop_shape",
                    "Please make sure that the first argument is a valid shape."
                );
            }
            let shape = std::mem::transmute::<u64, *mut strobe::ShapeEnum<'a>>(*shape_matlab_addr);

            strobe::drop_shape(shape);
        },
        "drop_texture" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:drop_texture",
                    "Please provide the texture that should be dropped."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:drop_texture",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:drop_texture",
                    "Please make sure that the first argument is a valid texture."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let texture_matlab_addr = mxGetData(p) as *const u64;
            if *(texture_matlab_addr.offset(1)) != StrobeTexture {
                genMatlabErrorMsg!(
                    "Strobe:drop_texture",
                    "Please make sure that the first argument is a valid texture."
                );
            }
            let texture =
                std::mem::transmute::<u64, *mut glium::texture::Texture2d>(*texture_matlab_addr);

            strobe::drop_texture(texture);
        },
        "drop_image" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:drop_image",
                    "Please provide the image that should be dropped."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:drop_image",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:drop_image",
                    "Please make sure that the first argument is a valid image."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let image_matlab_addr = mxGetData(p) as *const u64;
            if *(image_matlab_addr.offset(1)) != StrobeImage {
                genMatlabErrorMsg!(
                    "Strobe:drop_image",
                    "Please make sure that the first argument is a valid image."
                );
            }
            let image = std::mem::transmute::<u64, *mut image_ext::RgbaImage>(*image_matlab_addr);

            strobe::drop_image(image);
        },
        "drop_shader" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:drop_shader",
                    "Please provide the shader that should be dropped."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:drop_shader",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:drop_shader",
                    "Please make sure that the first argument is a valid shader."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let shader_matlab_addr = mxGetData(p) as *const u64;
            if *(shader_matlab_addr.offset(1)) != StrobeShader {
                genMatlabErrorMsg!(
                    "Strobe:drop_shader",
                    "Please make sure that the first argument is a valid shader."
                );
            }
            let shader =
                std::mem::transmute::<u64, *mut p5::shaders::ShaderInfo<'a>>(*shader_matlab_addr);

            strobe::drop_shader(shader);
        },
        "drop_fbo" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:drop_fbo",
                    "Please provide the FBO that should be dropped."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:drop_fbo",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:drop_fbo",
                    "Please make sure that the first argument is a valid FBO."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let fbo_matlab_addr = mxGetData(p) as *const u64;
            if *(fbo_matlab_addr.offset(1)) != StrobeFBO {
                genMatlabErrorMsg!(
                    "Strobe:drop_fbo",
                    "Please make sure that the first argument is a valid FBO."
                );
            }
            let fbo = std::mem::transmute::<u64, *mut glium::framebuffer::SimpleFrameBuffer>(
                *fbo_matlab_addr,
            );

            strobe::drop_fbo(fbo);
        },
        "drop_mould" => unsafe {
            if nrhs != 2 {
                genMatlabErrorMsg!(
                    "Strobe:drop_mould",
                    "Please provide the Mould that should be dropped."
                );
            }

            if nlhs != 0 {
                genMatlabErrorMsg!(
                    "Strobe:drop_mould",
                    "It does not make sense to have an output variable for this function.
                    Please remove the output variable on the left hand side of the equals
                    sign."
                );
            }

            if !(is_it_u64_w_type!(*(prhs.offset(1)))) {
                genMatlabErrorMsg!(
                    "Strobe:drop_mould",
                    "Please make sure that the first argument is a valid Mould."
                );
            }

            let p = *(prhs.offset(1)) as *const mxArray;
            let mould_matlab_addr = mxGetData(p) as *const u64;
            if *(mould_matlab_addr.offset(1)) != StrobeMould {
                genMatlabErrorMsg!(
                    "Strobe:drop_mould",
                    "Please make sure that the first argument is a valid Mould."
                );
            }
            let mould =
                std::mem::transmute::<u64, *mut strobe::MouldExtern<'a>>(*mould_matlab_addr);

            strobe::drop_mould(mould);
        },

        // everything else
        _ => unsafe { mexPrintf("not yet implemented, sorry...\n".as_ptr()) },
    }
}
