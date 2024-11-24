target/debug/jisho.exe: src/main.rs src/window/mod.rs src/window/mainwind.rs src/window/popupwind.rs resources/libjisho.a build.rs
	cargo build

run: target/debug/jisho.exe
	target/debug/jisho.exe

resources/libjisho.a: resources/jisho.res
	cvtres /MACHINE:x64 /OUT:resources/libjisho.a resources/jisho.res

resources/jisho.res: resources/jisho.rc resources/jisho.ico resources/jisho.exe.manifest src/resources.h
	rc /i "C:\Program Files (x86)\Windows Kits\10\Include\10.0.22000.0\um" /i "C:\Program Files (x86)\Windows Kits\10\Include\10.0.22000.0\shared" resources/jisho.rc