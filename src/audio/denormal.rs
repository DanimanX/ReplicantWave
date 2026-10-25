//! Denormal (subnormal) handling for the realtime audio thread.
//!
//! Feedback structures such as the reverb comb/allpass network and the delay
//! damping filter decay towards zero. Once their state lands in the subnormal
//! range, some CPUs fall back to microcode and a single sample can take
//! hundreds of cycles, producing periodic dropouts that sound exactly like
//! crackle. Enabling FTZ/DAZ on the audio thread removes the problem at the
//! source; the DSP code additionally flushes feedback state defensively.

/// Enable flush-to-zero and denormals-are-zero on the current thread.
#[inline]
pub fn flush_denormals_to_zero() {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let mut csr: u32 = 0;
        // stmxcsr/ldmxcsr are the stable replacement for the deprecated
        // _mm_getcsr/_mm_setcsr intrinsics.
        core::arch::asm!(
            "stmxcsr [{}]",
            in(reg) &mut csr,
            options(nostack, preserves_flags)
        );
        // Bit 15 = FTZ (flush to zero), bit 6 = DAZ (denormals are zero).
        csr |= (1 << 15) | (1 << 6);
        core::arch::asm!(
            "ldmxcsr [{}]",
            in(reg) &csr,
            options(nostack, preserves_flags)
        );
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        // FPCR bit 24 is the AArch64 flush-to-zero control.
        let mut fpcr: u64 = 0;
        core::arch::asm!("mrs {}, fpcr", out(reg) fpcr, options(nomem, nostack, preserves_flags));
        fpcr |= 1 << 24;
        core::arch::asm!("msr fpcr, {}", in(reg) fpcr, options(nomem, nostack, preserves_flags));
    }
}
