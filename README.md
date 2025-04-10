## yaxpeax-hexagon

[![crate](https://img.shields.io/crates/v/yaxpeax-hexagon.svg?logo=rust)](https://crates.io/crates/yaxpeax-hexagon)
[![documentation](https://docs.rs/yaxpeax-hexagon/badge.svg)](https://docs.rs/yaxpeax-hexagon)

Qualcomm Hexagon decoder implemented as part of the yaxpeax project, implementing traits provided by `yaxpeax-arch`.

support is good enough to make sense of many programs, but not complete:

- [x] instructions mentioned in the V62 and V73 manuals.
- [x] system instructions documented in V62 and earlier
- [ ] undocumented system instructions in V73 and later
- [ ] HVX (in any version)
- [ ] duplex instructions

between V67 and V73, Qualcomm decided to remove most mentions of the Hexagon
supervisor mode from their manuals. the LLVM target has had support for these
instructions since late 2023, so LLVM-derived disassemblers should support
them. it is not immediately clear to me that system instructions have the same
encodings or semantic on later architectures - i largely lack programs known to
target newer versions to validate that disassembly still looks reasonable.

Hexagon in real use seems to rely on a hypervisor (probably
Qualcomm-maintained? similar to their
[minivm](https://github.com/quic/hexagonMVM)?) which system instructions are
intended to support, then "User" and "Guest" modes which are more openly
documented in public manuals. none the less, `hexagonMVM` uses these
now-undocumented system instructions [for system register
management](https://github.com/quic/hexagonMVM/blob/db795a9/minivm.S#L259), TLB
management later on, traps, and so on. these system instructions are also
important to process to make sense of the entrypoints of in-the-wild Hexagon
firmware images.

### features

* `#[no_std]`
* exists (this is not the only Hexagon disassembler by any means)

### mirrors

the canonical copy of `yaxpeax-hexagon` is at [https://git.iximeow.net/yaxpeax-hexagon/](https://git.iximeow.net/yaxpeax-hexagon/).

`yaxpeax-hexagon` is also mirrored on GitHub at [https://www.github.com/iximeow/yaxpeax-hexagon](https://www.github.com/iximeow/yaxpeax-hexagon).

### see also

* [idp\_hexagon](https://github.com/n-o-o-n/idp_hexagon): IDA pro module for Hexagon. heavily derived from LLVM.
* [llvm](https://github.com/llvm/llvm-project/tree/e03f427/llvm/lib/Target/Hexagon)
* [r2hexagon](https://github.com/radareorg/r2hexagon): radare2's Hexagon disassembler. generated from manuals.
* [hexag00n](https://github.com/programa-stic/hexag00n): python-based Hexagon disassembler with IDA plugin
* [hexagon](https://github.com/gsmk/hexagon): another IDA pro processor module. wrapper for Sourcery CodeBench.
* [nogaxeh](https://github.com/ANSSI-FR/nogaxeh): another IDA pro processor module
* [rz-hexagon](https://github.com/rizinorg/rz-hexagon): Hexagon disassembler for rizin. generated from LLVM.

### changelog

a changelog across crate versions is maintained in the `CHANGELOG` file located in the repo, as well as [online](https://git.iximeow.net/yaxpeax-hexagon/tree/CHANGELOG).
