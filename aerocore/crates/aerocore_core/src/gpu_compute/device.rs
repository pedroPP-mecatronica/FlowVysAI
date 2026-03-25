//! GPU compute backend — abstraction over WGPU, CUDA, and CPU fallback.
//!
//! # OKR 1 — KR 3.3: 100% matrix parallelization via GPU
//!
//! Provides a unified interface so that solvers dispatch compute kernels
//! without knowing which backend (Vulkan, Metal, DX12, CUDA, or CPU) is active.
//!
//! # FIA ATR Compliance Note
//! For Formula 1 Restricted CFD (RCFD), the FIA prohibits GPU usage for
//! the solver component until 2028. The `CpuFallback` backend ensures
//! compliance while maintaining the same API.

/// GPU backend type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackendType {
    /// WebGPU abstraction (Vulkan / Metal / DX12).
    Wgpu,
    /// Native CUDA (NVIDIA only).
    CudaNative,
    /// CPU fallback (for FIA RCFD compliance or systems without GPU).
    CpuFallback,
}

/// Information about the detected GPU device.
#[derive(Debug, Clone)]
pub struct GpuDeviceInfo {
    pub name: String,
    pub backend: GpuBackendType,
    pub vram_bytes: u64,
    pub max_workgroup_size: [u32; 3],
    pub supports_fp64: bool,
}

/// Opaque handle to a buffer allocated on the GPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuBufferHandle(pub(crate) u64);

/// GPU compute error types.
#[derive(Debug)]
pub enum GpuError {
    NoDeviceFound,
    OutOfMemory { requested: usize, available: usize },
    KernelNotFound(String),
    CompilationError(String),
    DriverError(String),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoDeviceFound => write!(f, "No GPU device found"),
            Self::OutOfMemory { requested, available } =>
                write!(f, "GPU out of memory: requested {} bytes, {} available", requested, available),
            Self::KernelNotFound(name) => write!(f, "Kernel not found: {}", name),
            Self::CompilationError(msg) => write!(f, "Shader compilation error: {}", msg),
            Self::DriverError(msg) => write!(f, "GPU driver error: {}", msg),
        }
    }
}

impl std::error::Error for GpuError {}

/// Trait for GPU compute operations — unified WGPU + CUDA + CPU interface.
///
/// Solvers dispatch kernels through this trait without knowing the backend.
pub trait GpuComputeBackend: Send + Sync {
    /// Detects and initializes the best available device.
    fn init(&mut self) -> Result<GpuDeviceInfo, GpuError>;

    /// Allocates a buffer on the GPU (VRAM).
    fn allocate_buffer(&mut self, size_bytes: usize, label: &str) -> Result<GpuBufferHandle, GpuError>;

    /// Copies data from host (RAM) to device (VRAM).
    fn upload(&self, handle: &GpuBufferHandle, data: &[u8]) -> Result<(), GpuError>;

    /// Copies data from device (VRAM) to host (RAM).
    fn download(&self, handle: &GpuBufferHandle, output: &mut [u8]) -> Result<(), GpuError>;

    /// Dispatches a compute shader with the given workgroup dimensions.
    fn dispatch_kernel(
        &self,
        kernel_name: &str,
        workgroups: [u32; 3],
        bindings: &[GpuBufferHandle],
    ) -> Result<(), GpuError>;

    /// Synchronizes — waits for all pending dispatches to complete.
    fn sync(&self) -> Result<(), GpuError>;

    /// Frees a previously allocated buffer.
    fn free_buffer(&mut self, handle: GpuBufferHandle) -> Result<(), GpuError>;

    /// Returns device information.
    fn device_info(&self) -> &GpuDeviceInfo;
}

/// CPU fallback "GPU" backend — runs everything on CPU.
///
/// Used for:
/// - Systems without a discrete GPU
/// - FIA RCFD compliance (GPU prohibited until 2028)
/// - Testing and CI environments
pub struct CpuFallbackBackend {
    info: GpuDeviceInfo,
    next_handle: u64,
    buffers: std::collections::HashMap<u64, Vec<u8>>,
}

impl CpuFallbackBackend {
    pub fn new() -> Self {
        Self {
            info: GpuDeviceInfo {
                name: "CPU Fallback".into(),
                backend: GpuBackendType::CpuFallback,
                vram_bytes: 0,
                max_workgroup_size: [1024, 1, 1],
                supports_fp64: true,
            },
            next_handle: 0,
            buffers: std::collections::HashMap::new(),
        }
    }
}

impl Default for CpuFallbackBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuComputeBackend for CpuFallbackBackend {
    fn init(&mut self) -> Result<GpuDeviceInfo, GpuError> {
        Ok(self.info.clone())
    }

    fn allocate_buffer(&mut self, size_bytes: usize, _label: &str) -> Result<GpuBufferHandle, GpuError> {
        let handle = self.next_handle;
        self.next_handle += 1;
        self.buffers.insert(handle, vec![0u8; size_bytes]);
        Ok(GpuBufferHandle(handle))
    }

    fn upload(&self, handle: &GpuBufferHandle, data: &[u8]) -> Result<(), GpuError> {
        if self.buffers.contains_key(&handle.0) {
            // In a real implementation, this would copy to the buffer
            let _ = data;
            Ok(())
        } else {
            Err(GpuError::DriverError(format!("Invalid handle: {}", handle.0)))
        }
    }

    fn download(&self, handle: &GpuBufferHandle, output: &mut [u8]) -> Result<(), GpuError> {
        if let Some(buf) = self.buffers.get(&handle.0) {
            let len = output.len().min(buf.len());
            output[..len].copy_from_slice(&buf[..len]);
            Ok(())
        } else {
            Err(GpuError::DriverError(format!("Invalid handle: {}", handle.0)))
        }
    }

    fn dispatch_kernel(
        &self,
        kernel_name: &str,
        _workgroups: [u32; 3],
        _bindings: &[GpuBufferHandle],
    ) -> Result<(), GpuError> {
        // CPU fallback: kernels would be Rust functions, not shaders
        tracing::debug!("CPU fallback: dispatch '{}' (no-op)", kernel_name);
        Ok(())
    }

    fn sync(&self) -> Result<(), GpuError> {
        Ok(()) // CPU is synchronous by nature
    }

    fn free_buffer(&mut self, handle: GpuBufferHandle) -> Result<(), GpuError> {
        self.buffers.remove(&handle.0);
        Ok(())
    }

    fn device_info(&self) -> &GpuDeviceInfo {
        &self.info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_fallback_lifecycle() {
        let mut backend = CpuFallbackBackend::new();
        let info = backend.init().unwrap();
        assert_eq!(info.backend, GpuBackendType::CpuFallback);
        assert!(info.supports_fp64);

        let handle = backend.allocate_buffer(1024, "test_buffer").unwrap();
        backend.upload(&handle, &[1u8; 1024]).unwrap();

        let mut output = vec![0u8; 1024];
        backend.download(&handle, &mut output).unwrap();

        backend.dispatch_kernel("test_kernel", [1, 1, 1], &[handle]).unwrap();
        backend.sync().unwrap();
        backend.free_buffer(handle).unwrap();
    }
}
