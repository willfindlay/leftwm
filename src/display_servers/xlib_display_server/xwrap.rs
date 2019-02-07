use x11_dl::xlib;
use std::os::raw::{ c_int, c_uint, c_char, c_long };
use std::ffi::{ CString };
use std::ptr;
use super::utils;
use std::slice;


pub struct XWrap{
    xlib: xlib::Xlib,
    display: *mut xlib::Display
}


impl XWrap {

    pub fn new() -> XWrap {
        let xlib = xlib::Xlib::open().unwrap();
        let display = unsafe{ (xlib.XOpenDisplay)(ptr::null()) };
        assert!(!display.is_null(), "Null pointer in display");
        XWrap{
            xlib: xlib,
            display: display
        }
    }


    //returns all the screens the display
    pub fn get_screens(&self) -> Vec<*mut xlib::Screen> {
        let mut screens = Vec::new();
        let screen_count = unsafe{ (self.xlib.XScreenCount)(self.display) };
        for screen_num in 0..(screen_count) {
            let screen = unsafe{ (self.xlib.XScreenOfDisplay)(self.display, screen_num) };
            screens.push( screen );
        }
        screens
    }

    //returns all the roots the display
    pub fn get_roots(&self) -> Vec<xlib::Window> {
        return self.get_screens().into_iter().map(|s| {
        unsafe{ (self.xlib.XRootWindowOfScreen)(s) }
        }).collect();
    }

    //returns all the windows under a root windows
    pub fn get_windows_for_root<'w>(&self, root: xlib::Window) -> Result<&'w[xlib::Window], String>
    {
        unsafe{
            let mut root_return: xlib::Window = std::mem::zeroed();
            let mut parent_return: xlib::Window = std::mem::zeroed();
            let mut array: *mut xlib::Window = std::mem::zeroed();
            let mut length: c_uint = std::mem::zeroed();
            let status : xlib::Status = (self.xlib.XQueryTree)(self.display,
                                                          root, 
                                                          &mut root_return,
                                                          &mut parent_return, 
                                                          &mut array,
                                                          &mut length);
            let windows: &[xlib::Window] = slice::from_raw_parts(array, length as usize);
            match status {
                0 /* XcmsFailure */ => { Err("Could not load list of windows".to_string() ) }
                1 /* XcmsSuccess */ => { Ok(windows) }
                2 /* XcmsSuccessWithCompression */ => { Ok(windows) }
                _ => { Err("Unknown return status".to_string() ) }
            }
        }
    }


    pub fn get_all_windows(&self) -> Result<Vec<xlib::Window>, String> {
        let mut all = Vec::new();
        for root in self.get_roots() {
            match self.get_windows_for_root(root) {
                Ok(some_windows) => {
                    for w in some_windows {
                        all.push( *w );
                    }
                }
                Err(err) => { return Err(err) }
            }
        }
        return Ok(all);
    } 

    pub fn get_window_attrs(&self, window: xlib::Window) -> Result<xlib::XWindowAttributes, ()> {
        let mut attrs: xlib::XWindowAttributes = unsafe{ std::mem::zeroed() };
        let status = unsafe { (self.xlib.XGetWindowAttributes)(self.display, window, &mut attrs) };
        if status == 0 { return Err(()) }
        return Ok(attrs);
    }


    pub fn update_window(&self, window: &utils::window::Window ) {
        use utils::window::WindowHandle;
        let mut changes =xlib::XWindowChanges{
            x: window.x,
            y: window.y,
            width: window.width,
            height: window.height,
            border_width: window.border,
            sibling: 0,    //not unlocked
            stack_mode: 0, //not unlocked
        };
        println!("in xwrap update window: {:#?} ", changes);
        if let WindowHandle::XlibHandle(h) = window.handle {
            let unlock = xlib::CWX | xlib::CWY | xlib::CWWidth | xlib::CWHeight | xlib::CWBorderWidth;
            unsafe {
                (self.xlib.XConfigureWindow)(self.display, h, unlock as u32, &mut changes);
                (self.xlib.XSync)(self.display, 0); 
            }
        }
    }


    pub fn get_transient_for(&self, window: xlib::Window) -> Option<xlib::Window> {
        unsafe{
            let mut transient: xlib::Window = std::mem::zeroed();
            let status :c_int = (self.xlib.XGetTransientForHint)(
                self.display,
                window,
                &mut transient);
            if status > 0 {
                Some( transient )
            }
            else {
                None
            }
        }
    }

    pub fn get_window_name(&self, window: xlib::Window) -> Option<String> {
        let c_string = unsafe{
            let mut ptr : *mut c_char = std::mem::zeroed();
            let status :c_int = (self.xlib.XFetchName)(
                self.display, 
                window, 
                &mut ptr );
            if status == 0 {  return None; }
            CString::from_raw(ptr)
        };
        match c_string.into_string() {
            Ok(s) => { return Some(s) }
            Err(_) => { return None }
        }
    }


    ////get the WMName of a window
    //pub fn get_window_title(&self, window: xlib::Window) -> Result<String, ()> {
    //    unsafe{
    //        let mut ptr : *mut *mut c_char = std::mem::zeroed();
    //        let mut ptr_len: c_int = 0;
    //        let mut text_prop: xlib::XTextProperty = std::mem::zeroed();
    //        let status :c_int = (self.xlib.XGetTextProperty)(
    //            self.display, 
    //            window, 
    //            &mut text_prop,
    //            2);
    //        if status == 0 { return Err( () ) }
    //        (self.xlib.XTextPropertyToStringList)( 
    //            &mut text_prop, 
    //            &mut ptr, 
    //            &mut ptr_len );
    //        let raw: &[*mut c_char] = slice::from_raw_parts(ptr, ptr_len as usize);
    //        for i in 0..ptr_len {
    //            if let Ok(s) = CString::from_raw(*ptr).into_string() {
    //                return Ok(s)
    //            }
    //        }
    //    };
    //    return Err(())
    //}


    ////get the WMName of a window
    //pub fn get_wmname(&self, window: xlib::Window) -> Result<String, ()> {
    //    unsafe{
    //        let mut ptr : *mut *mut c_char = std::mem::zeroed();
    //        let mut ptr_len: c_int = 0;
    //        let mut text_prop: xlib::XTextProperty = std::mem::zeroed();
    //        let status :c_int = (self.xlib.XGetWMName)(
    //            self.display, 
    //            window, 
    //            &mut text_prop );
    //        if status == 0 { return Err( () ) }
    //        (self.xlib.XTextPropertyToStringList)( 
    //            &mut text_prop, 
    //            &mut ptr, 
    //            &mut ptr_len );
    //        let raw: &[*mut c_char] = slice::from_raw_parts(ptr, ptr_len as usize);
    //        for i in 0..ptr_len {
    //            if let Ok(s) = CString::from_raw(*ptr).into_string() {
    //                return Ok(s)
    //            }
    //        }
    //    };
    //    return Err(())
    //}


    pub fn subscribe_to_event(&self, window: xlib::Window, mask: c_long){
        //let mut attrs: xlib::XSetWindowAttributes = unsafe{ std::mem::uninitialized() };
        //attrs.event_mask = *mask;
        //attrs.cursor = 0;
        unsafe{
            //let unlock = xlib::CWEventMask | xlib::CWCursor;
            //(self.xlib.XChangeWindowAttributes)(self.display, window, unlock, &mut attrs);
            (self.xlib.XSelectInput)(self.display, window, mask);
        }
    }


    //pub fn manage_window(&self, window: xlib::Window){
    //    let mask = xlib::EnterWindowMask |
    //        xlib::FocusChangeMask |
    //        xlib::PropertyChangeMask |
    //        xlib::StructureNotifyMask;
    //    self.subscribe_to_event(window,mask);
    //    unsafe{ (self.xlib.XMapWindow)(self.display, window) };
    //}

    pub fn init(&self){
        let root_event_mask: c_long = 
            xlib::ButtonPressMask
            | xlib::SubstructureRedirectMask
            | xlib::SubstructureNotifyMask
            | xlib::PointerMotionMask
            | xlib::EnterWindowMask
            | xlib::LeaveWindowMask
            | xlib::StructureNotifyMask
            | xlib::PropertyChangeMask;
        for root in self.get_roots() {
            self.subscribe_to_event(root, root_event_mask);
        }
        unsafe { (self.xlib.XSync)(self.display, 0); }
    }



    pub fn get_next_event(&self) -> xlib::XEvent {
        let mut event: xlib::XEvent = unsafe{ std::mem::uninitialized() };
        unsafe{
            (self.xlib.XNextEvent)(self.display, &mut event);
        };
        return event;
    }



}







