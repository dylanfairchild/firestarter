use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::ValidateRect, Win32::Graphics::Gdi::CreateFontA, Win32::Graphics::Gdi::FW_DONTCARE,
    Win32::Graphics::Gdi::ANSI_CHARSET, Win32::Graphics::Gdi::OUT_DEFAULT_PRECIS, Win32::Graphics::Gdi::CLIP_DEFAULT_PRECIS,
    Win32::Graphics::Gdi::DEFAULT_QUALITY, Win32::Graphics::Gdi::FF_DONTCARE,
    Win32::System::LibraryLoader::GetModuleHandleA, Win32::UI::Controls::*,
    Win32::UI::WindowsAndMessaging::*,
};

use reqwest;
use std::io::Read;
use std::thread;

static WINDOW_TITLE: &str = "firestarter";
static INFO_STRING: &str = "This is small boostrapping application to launch an application from a network resource. Please be patient while your application is downloaded and launched.";
static TARGET_STRING_PRE: &str = "Downloading and executing the following file: ";
static TARGET_STRING: &str =
    "https://wddashboarddownloads.wdc.com/wdDashboard/DashboardSetupSA.exe";

static mut PROGRESS: HWND = HWND(0);
static mut PROGRESS_LABEL: HWND = HWND(0);

fn playing_with_reqwest() {
    let mut resp = reqwest::blocking::get(TARGET_STRING).unwrap();

    let mut some_bytes = vec![0; 1024 * 1024 * 1024]; // Read up to 1MiB chunks
    let whole_size = resp.content_length().ok_or("No value").unwrap() as usize;
    let mut sum = 0;
    let mut last_label = String::new();
    while sum < whole_size {
        let read = resp.read(&mut some_bytes).unwrap();
        if read == 0 {
            break;
        }

        sum += read;

        let pct = (sum as f32 / whole_size as f32) * 100.0;
        let pct_str = pct.round().to_string() + "%";
        unsafe {
            SendMessageA(PROGRESS, PBM_SETPOS, WPARAM(pct as usize), LPARAM(0));
            if pct_str != last_label {
                last_label = pct_str.clone();
                SetWindowTextA(PROGRESS_LABEL, pct_str);
            }
        }

        println!("Read {}/{} bytes -- {} this xfer", sum, whole_size, read);
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let normal_font;
    let bold_font;

    unsafe {
        InitCommonControls();

        let instance = GetModuleHandleA(None)?;
        assert!(!instance.is_invalid(), "invalid instance");

        let wc = WNDCLASSEXA {
            cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance,
            lpszClassName: PCSTR(b"window\0".as_ptr()),
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassExA(&wc);
        assert!(atom != 0);

        let screenx = GetSystemMetrics(SM_CXSCREEN);
        let screeny = GetSystemMetrics(SM_CYSCREEN);

        normal_font = CreateFontA(
            0,
            0,
            0,
            0,
            FW_DONTCARE.try_into().unwrap(),
            0,
            0,
            0,
            ANSI_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            DEFAULT_QUALITY,
            FF_DONTCARE,
            "Segoe UI"
        );

        let wnd = CreateWindowExA(
            Default::default(),
            "window",
            WINDOW_TITLE,
            WS_OVERLAPPEDWINDOW | WS_VISIBLE | WINDOW_STYLE(PBS_MARQUEE),
            screenx / 2 - 300,
            screeny / 2 - 100,
            600,
            200,
            None,
            None,
            instance,
            std::ptr::null(),
        );

        let info = CreateWindowExA(
            Default::default(),
            "STATIC",
            "",
            WS_CHILD | WS_VISIBLE,
            10,
            10,
            600,
            40,
            wnd,
            None,
            instance,
            std::ptr::null(),
        );
        SendMessageA(info, WM_SETFONT, WPARAM(normal_font.0 as usize), LPARAM(0));
        SetWindowTextA(info, INFO_STRING);

        let target = CreateWindowExA(
            Default::default(),
            "STATIC",
            "",
            WS_CHILD | WS_VISIBLE,
            10,
            60,
            600,
            40,
            wnd,
            None,
            instance,
            std::ptr::null(),
        );
        SendMessageA(target, WM_SETFONT, WPARAM(normal_font.0 as usize), LPARAM(0));
        SetWindowTextA(target, TARGET_STRING_PRE.to_string() + TARGET_STRING);

        PROGRESS = CreateWindowExA(
            Default::default(),
            PROGRESS_CLASS,
            "",
            WS_CHILD | WS_VISIBLE,
            10,
            110,
            500,
            20,
            wnd,
            None,
            instance,
            std::ptr::null(),
        );
        SendMessageA(PROGRESS, PBM_SETRANGE, WPARAM(0), LPARAM(100 << 16));
        SendMessageA(PROGRESS, PBM_SETMARQUEE, WPARAM(1), LPARAM(0));
        SendMessageA(PROGRESS, PBM_SETPOS, WPARAM(0), LPARAM(0)); // Set the position so it draws

        PROGRESS_LABEL = CreateWindowExA(
            Default::default(),
            "STATIC",
            "",
            WS_CHILD | WS_VISIBLE,
            530,
            110,
            50,
            20,
            wnd,
            None,
            instance,
            std::ptr::null(),
        );
        SendMessageA(PROGRESS_LABEL, WM_SETFONT, WPARAM(normal_font.0 as usize), LPARAM(0));
        SetWindowTextA(PROGRESS_LABEL, "0%");
    }

    let thr = thread::spawn(move || {
        playing_with_reqwest();
    });

    unsafe {
        let mut message = MSG::default();
        while GetMessageA(&mut message, HWND(0), 0, 0).into() {
            DispatchMessageA(&message);
        }
    }

    Ok(())
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message as u32 {
            WM_PAINT => {
                println!("WM_PAINT");
                ValidateRect(window, std::ptr::null());
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_CTLCOLORSTATIC => {
                println!("WM_CTLCOLORSTATIC");
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
