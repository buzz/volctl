# Changelog
All notable changes to this project will be documented in this file.

## [1.0.0](https://github.com/buzz/volctl/compare/v0.9.5...v1.0.0) - 2026-05-23

### ♻️ Refactoring
- trim Pulse wrapper to required functionality by [@buzz](https://github.com/buzz) ([86bec25](https://github.com/buzz/volctl/commit/86bec25ec70f377b6c4864f297ef6cd665cd9385))
- replace custom Shared type with Rc<RefCell<T>> by [@buzz](https://github.com/buzz) ([d12c110](https://github.com/buzz/volctl/commit/d12c110f67b5196481944d322a43c821e4bb8f3b))
- replace std::sync::mpsc with async_channel by [@buzz](https://github.com/buzz) ([7565d4f](https://github.com/buzz/volctl/commit/7565d4fccb0724c22ae2ca367ac610335aaae246))
- modularize app and mixer window into submodules by [@buzz](https://github.com/buzz) ([1c0ef07](https://github.com/buzz/volctl/commit/1c0ef07c3f3200c0f28c759ed960f08d17816e9c))
- clean up tray callback logic by [@buzz](https://github.com/buzz) ([97de59e](https://github.com/buzz/volctl/commit/97de59ed609f16a00eb64ed1ad1ba2386c01d365))
- modernize pulseaudio wrapper and improve safety by [@buzz](https://github.com/buzz) ([694715d](https://github.com/buzz/volctl/commit/694715df83cffd995437fa7a6ec522c94a3be547))
- close/recreate mixer window on toggle by [@buzz](https://github.com/buzz) ([6b8da56](https://github.com/buzz/volctl/commit/6b8da56885a94384eb70f236f1cdda7f05a5ab0c))
- remove x11rb by reusing GDK's X11 connection by [@buzz](https://github.com/buzz) ([4aa2e79](https://github.com/buzz/volctl/commit/4aa2e79d92e43496dd1c966d71ff0df767c0142f))
- replace x11rb with raw xlib via shared X11Context by [@buzz](https://github.com/buzz) ([44e351d](https://github.com/buzz/volctl/commit/44e351dd35d832c9455b830af675b38ee2037720))
- consolidate shared X11 utilities and fix mixer window placement by [@buzz](https://github.com/buzz) ([7001702](https://github.com/buzz/volctl/commit/70017021ee2a5bd1cc5f7191f51a6f2b7143f33d))
- add consistent padding to mixer window by [@buzz](https://github.com/buzz) ([ce6d9c7](https://github.com/buzz/volctl/commit/ce6d9c7ff554e23acd0cc5212a2c3b9dc967964e))
- remove prefer-gtksi XEmbed preference option by [@buzz](https://github.com/buzz) ([c37dec4](https://github.com/buzz/volctl/commit/c37dec41bb0717b8b6eba154034611d4b2f7d5a9))
- implement structured error handling with tracing by [@buzz](https://github.com/buzz) ([e5bce3d](https://github.com/buzz/volctl/commit/e5bce3d9c2cabd68173853a0aee590c91b974823))
- wire peak updates and auto-toggle VU monitoring by [@buzz](https://github.com/buzz) ([7942661](https://github.com/buzz/volctl/commit/7942661438a46171025c214963f35fa72d65e2b5))
- move #[link(name = "Xfixes")] to crate root by [@buzz](https://github.com/buzz) ([0c45596](https://github.com/buzz/volctl/commit/0c45596cd3bc265506d3f9959b10cb4fa4e37314))
- make X11Context a zero-sized type by [@buzz](https://github.com/buzz) ([ed8be01](https://github.com/buzz/volctl/commit/ed8be0103e64b10f94d731a3a10e10c837b9ac57))
- convert show_about and show_prefs to associated functions by [@buzz](https://github.com/buzz) ([429efae](https://github.com/buzz/volctl/commit/429efae6c51ff617c29c52f87648f1bb883429ba))
- address low-priority code review findings by [@buzz](https://github.com/buzz) ([ccbadbf](https://github.com/buzz/volctl/commit/ccbadbf49cd79e96473e80d955dae295c7cd03d2))
- split module into types, monitor, and controller by [@buzz](https://github.com/buzz) ([2692768](https://github.com/buzz/volctl/commit/2692768094674a12fc82cd43fa3a9892e6f6c383))

### ⚡ Performance
- pass AtomCollection to set_window_type instead of recreating it by [@buzz](https://github.com/buzz) ([26ec509](https://github.com/buzz/volctl/commit/26ec5097d7f07dec44c0b8164b1e66d346edac4b))
- add minimum decay floor to peak values for smoother VU transitions by [@buzz](https://github.com/buzz) ([6f1e4dd](https://github.com/buzz/volctl/commit/6f1e4ddab777c8e27f2cbe67a7ee8370ce321e46))

### ✨ Features
- scaffold initial project structure by [@buzz](https://github.com/buzz) ([6b97ae6](https://github.com/buzz/volctl/commit/6b97ae6f59d445c6a850a2ae42c94b878198f0a0))
- add UI modules (tray, mixer window, X11, Wayland) by [@buzz](https://github.com/buzz) ([cfdad27](https://github.com/buzz/volctl/commit/cfdad278333ab284b4026c5f55e4f5eb6bf9d34f))
- implement GtkApplication subclass by [@buzz](https://github.com/buzz) ([10dc6e6](https://github.com/buzz/volctl/commit/10dc6e6d50717b1c54865edcae29b58686727e89))
- implement GtkWindow subclass by [@buzz](https://github.com/buzz) ([784e0ad](https://github.com/buzz/volctl/commit/784e0ad6c9bfa54e82856e8b3224e01fdcf389bf))
- add application lifecycle and state management by [@buzz](https://github.com/buzz) ([806b718](https://github.com/buzz/volctl/commit/806b71842112f8b623c4f53ce096dc472cfda6bc))
- add PulseAudio integration by [@buzz](https://github.com/buzz) ([028b51e](https://github.com/buzz/volctl/commit/028b51e41375526875fc004e3f6266d88665fc3c))
- implement dynamic status icon updates by [@buzz](https://github.com/buzz) ([2086c81](https://github.com/buzz/volctl/commit/2086c814c102012410c9ea236b0b7cfe75289485))
- add scroll wheel and tooltip support by [@buzz](https://github.com/buzz) ([632807d](https://github.com/buzz/volctl/commit/632807d8907cbfb0b4a2e669d4a0fc26976a35d8))
- add menu items and active sink mute toggle by [@buzz](https://github.com/buzz) ([5d20e87](https://github.com/buzz/volctl/commit/5d20e87e9b8ec7131e0f298b627a7112f4717698))
- dynamically add/remove/update volume scales by [@buzz](https://github.com/buzz) ([d7cfd07](https://github.com/buzz/volctl/commit/d7cfd07c45627dcd631335ecd6e13a0f5f79d9fc))
- add error handling by [@buzz](https://github.com/buzz) ([0838047](https://github.com/buzz/volctl/commit/0838047144df4dcf6e91f8f0f731feacea594a14))
- add preferences window by [@buzz](https://github.com/buzz) ([27fe434](https://github.com/buzz/volctl/commit/27fe434f096364518f65b315a871c309d5675952))
- add OSD (X11) by [@buzz](https://github.com/buzz) ([5b0faac](https://github.com/buzz/volctl/commit/5b0faac46b1aedf5da6c5a4edb66e6ba6a6be577))
- add OSD (Wayland) by [@buzz](https://github.com/buzz) ([f9bbabe](https://github.com/buzz/volctl/commit/f9bbabeb3b92910218b6cfe83456d154efc2c6ce))
- add about dialog by [@buzz](https://github.com/buzz) ([ace652d](https://github.com/buzz/volctl/commit/ace652d3432b318c34613e7eab2d279f0efaa640))
- position mixer window relative to tray anchor using screen quadrant by [@buzz](https://github.com/buzz) ([90ff0f7](https://github.com/buzz/volctl/commit/90ff0f7d16a9f8f7e8737b2c14ff8a2714d3c268))
- add auto-close timeout with mouse hover pause by [@buzz](https://github.com/buzz) ([9e725e9](https://github.com/buzz/volctl/commit/9e725e99fbf5d0e61bff7ca808a47b1117763f57))
- enable optional extra volume (up to 150%) by [@buzz](https://github.com/buzz) ([492f307](https://github.com/buzz/volctl/commit/492f3071cc4727e8f5a895bb31c45254c428845a))
- implement external mixer launch and wire secondary tray action by [@buzz](https://github.com/buzz) ([368192d](https://github.com/buzz/volctl/commit/368192defb75189bb6d8ccc77fb37f8ea1867c7f))
- add setting to toggle percentage display on volume scale by [@buzz](https://github.com/buzz) ([1590c75](https://github.com/buzz/volctl/commit/1590c756656999811781e17885391b1517e0ab9f))
- add separator between audio interfaces and applications in mixer by [@buzz](https://github.com/buzz) ([d0be1eb](https://github.com/buzz/volctl/commit/d0be1eb5da77031f67986fd51852b417f062d3aa))
- add peak level monitoring with smooth decay and VU meter by [@buzz](https://github.com/buzz) ([afdd10e](https://github.com/buzz/volctl/commit/afdd10e1f7af1f827d523694e04061122f9791c3))
- improve tooltips with rich formatting and media info by [@buzz](https://github.com/buzz) ([d04a0f8](https://github.com/buzz/volctl/commit/d04a0f8e1dda2fa8dc7c2ead5ce6eaafab275bc5))
- color VU meter fill red on clipping (peak >= 1.0) by [@buzz](https://github.com/buzz) ([03d3b3f](https://github.com/buzz/volctl/commit/03d3b3fa40cdbdf4bc5b03906a779532b4c9a9f2))
- add Wayland mixer window placement settings by [@buzz](https://github.com/buzz) ([64f26cd](https://github.com/buzz/volctl/commit/64f26cd1e13acca0d63523b72fc7ae03dfa3810d))
- add configurable margin settings for OSD and mixer window by [@buzz](https://github.com/buzz) ([69a067f](https://github.com/buzz/volctl/commit/69a067f810c4c1bd41fcab92ca2906787ce3ed5a))
- add toggle to enable/disable fade animations by [@buzz](https://github.com/buzz) ([5f83f56](https://github.com/buzz/volctl/commit/5f83f56bd7e213c3ddf54fcafabe7070c044f8a1))

### 🐛 Bug Fixes
- set proper window hints and properties by [@buzz](https://github.com/buzz) ([f4dc473](https://github.com/buzz/volctl/commit/f4dc473817d93ad3b0d691158f5d54f9806325e0))
- break tray message loop on application quit by [@buzz](https://github.com/buzz) ([1caa34e](https://github.com/buzz/volctl/commit/1caa34e65e64ae2c5aefb579cc4f9ac002c5b408))
- persist/restore settings correctly, update OSD settings controls disable state by [@buzz](https://github.com/buzz) ([2ca8bd1](https://github.com/buzz/volctl/commit/2ca8bd1057e58553bd79b895db521a43d4ec1820))
- wire VolumeScale signals to correct PulseAudio stream by [@buzz](https://github.com/buzz) ([b52b607](https://github.com/buzz/volctl/commit/b52b6079a9c8d3956f6e670712ba9efdbaeef7b7))
- fix icon resolution for sink and sink input streams by [@buzz](https://github.com/buzz) ([7cae177](https://github.com/buzz/volctl/commit/7cae1775348ec4fb2d0f65cc50d18d5611d76498))
- prevent (0,0) flicker and duplicate window race on X11 by [@buzz](https://github.com/buzz) ([fbc3f33](https://github.com/buzz/volctl/commit/fbc3f33bc87b67c35e148897442263ece4671742))
- disable text selection on labels in about dialog by [@buzz](https://github.com/buzz) ([149a7a3](https://github.com/buzz/volctl/commit/149a7a37c57e67f6f85d5c3e4e82510be2515e79))
- prevent premature operation drop and remove unnecessary mainloop locking by [@buzz](https://github.com/buzz) ([cca5b2b](https://github.com/buzz/volctl/commit/cca5b2bfad3beacfeb96422a6deb3a472732ee35))
- remove std::mem::forget on Operation to fix memory leak by [@buzz](https://github.com/buzz) ([d28c7f2](https://github.com/buzz/volctl/commit/d28c7f20d46931039260c86d8dc7e274b68a0db5))
- install monitor stream callbacks before connect_record by [@buzz](https://github.com/buzz) ([6977ded](https://github.com/buzz/volctl/commit/6977ded951ee49bbe24cf4468fb0ab2365d58206))
- align `allow_extra_volume` default with GSettings schema by [@buzz](https://github.com/buzz) ([e7374e6](https://github.com/buzz/volctl/commit/e7374e6651050606410fdacebecc945b34e491c5))
- store and cancel the periodic update timer on shutdown by [@buzz](https://github.com/buzz) ([4118e60](https://github.com/buzz/volctl/commit/4118e60781fd1fb96981f618e76bc852a699f0e0))
- associate window with Application parent by [@buzz](https://github.com/buzz) ([8fc950e](https://github.com/buzz/volctl/commit/8fc950e873c9979cd30f1eb6b91ccad10f01d119))
- associate OSD window with Application parent by [@buzz](https://github.com/buzz) ([4b701e6](https://github.com/buzz/volctl/commit/4b701e60fabe6c15aa96163dd0cd8dc8f1f2180a))
- set opposite anchors for center positioning on Wayland by [@buzz](https://github.com/buzz) ([823e547](https://github.com/buzz/volctl/commit/823e547f081573c0e1c1ef6ddce04a674bcb1e78))
- move tooltip text to title field and remove markup by [@buzz](https://github.com/buzz) ([383fd3f](https://github.com/buzz/volctl/commit/383fd3f2d5c64ca2d684343b08dcac60c6a88909))
- fix layer-shell center positioning on Wayland by [@buzz](https://github.com/buzz) ([1aa2c84](https://github.com/buzz/volctl/commit/1aa2c84e2d696ac12e66734ed22923137c418d5b))
- set explicit layer-shell namespaces for OSD and mixer window by [@buzz](https://github.com/buzz) ([e5e18f4](https://github.com/buzz/volctl/commit/e5e18f40a3f60df030f9c6371f2b67fbfed82ff0))

### 💅 Styling
- add CSS styling for mute toggle button by [@buzz](https://github.com/buzz) ([fcb83ac](https://github.com/buzz/volctl/commit/fcb83ac73a3be82dc5884e386e28036de771c75a))

### 📚 Documentation
- add LICENSE.txt by [@buzz](https://github.com/buzz) ([99cac0f](https://github.com/buzz/volctl/commit/99cac0fe2278f9d802122cc936cf14461177a7e9))
- add README.md by [@buzz](https://github.com/buzz) ([ef3badf](https://github.com/buzz/volctl/commit/ef3badf336e0378840c5477749bc62745a559871))

### 📦 Dependencies
- upgrade ksni to 0.3.4 by [@buzz](https://github.com/buzz) ([b2f2d10](https://github.com/buzz/volctl/commit/b2f2d1025164c8bd3c3f29047e11f1e924effb72))
- cargo update by [@buzz](https://github.com/buzz) ([d0617a9](https://github.com/buzz/volctl/commit/d0617a99b149fe10b1ac3e850e20e96a1e6eee52))

### 🔀 Other Changes
- improve scale controls with tick marks and label alignment by [@buzz](https://github.com/buzz) ([438b842](https://github.com/buzz/volctl/commit/438b842a89cf6ce666269c15087b18c089bd8344))

### 🔧 Miscellaneous Chores
- enable Clippy linter in VS Code settings by [@buzz](https://github.com/buzz) ([d326af0](https://github.com/buzz/volctl/commit/d326af0854e6440642c47fd72d0cd468a1cc6535))
- fix Clippy warnings by [@buzz](https://github.com/buzz) ([a2305c0](https://github.com/buzz/volctl/commit/a2305c048c9f0cd1622372f23407d78f1013deb1))
- make libpulse-binding optional via feature flag by [@buzz](https://github.com/buzz) ([0cc205b](https://github.com/buzz/volctl/commit/0cc205b56a8b2033a4db90924774c97e7d459848))
- define settings key constants by [@buzz](https://github.com/buzz) ([93a1e1a](https://github.com/buzz/volctl/commit/93a1e1a04e36ef565502f207c4311ae85fe26928))
- upgrade dependencies by [@buzz](https://github.com/buzz) ([4bdeed8](https://github.com/buzz/volctl/commit/4bdeed8d773fc75477129f6e40a4b31c437f0478))
- add GSettings schema and desktop entry files by [@buzz](https://github.com/buzz) ([9545134](https://github.com/buzz/volctl/commit/9545134949250f7d1afbc2f963a619a981ecf7bb))
- upgrade rust edition to 2024 and fix clippy warnings by [@buzz](https://github.com/buzz) ([d41dfec](https://github.com/buzz/volctl/commit/d41dfece6b910a4afc3cbd4823edd7b9658425d4))
- add package metadata to Cargo.toml by [@buzz](https://github.com/buzz) ([0d96484](https://github.com/buzz/volctl/commit/0d96484ee8134750ee85a3c6a534714027d0e506))
- integrate git-cliff for changelog generation ([bdf3882](https://github.com/buzz/volctl/commit/bdf3882db4d36de69864c9d1a806b2e5fa729ed1))

---

> **Note:** Previous changes can be found in the [`legacy-python`](https://github.com/buzz/volctl/tree/legacy-python) branch.
