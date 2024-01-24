use std::ffi::{self, CStr};

use ash::vk;

//

pub struct Backend {
    // instance: ash::Instance,
    // gpu: vk::PhysicalDevice,
}

//

#[derive(Debug, Clone, Copy)]
pub struct VulkanLoadError;

//

pub fn new_backend() -> Result<Backend, VulkanLoadError> {
    let entry: ash::Entry = unsafe { ash::Entry::load() }.map_err(|err| {
        tracing::error!("vulkan cannot be loaded: {err}");
        VulkanLoadError
    })?;

    tracing::debug!("vulkan entry loaded");

    static ENGINE_NAME: &CStr =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"github.com/xor-bits/vk-ml\0") };
    static ENGINE_VERS: u32 = vk::make_api_version(1, 0, 1, 0);

    let app_info = vk::ApplicationInfo::builder()
        .application_name(ENGINE_NAME)
        .engine_name(ENGINE_NAME)
        .api_version(vk::API_VERSION_1_3)
        .application_version(ENGINE_VERS)
        .engine_version(ENGINE_VERS);
    let info = vk::InstanceCreateInfo::builder().application_info(&app_info);
    let instance: ash::Instance = unsafe { entry.create_instance(&info, None) }.map_err(|err| {
        tracing::error!("vulkan instance cannot be created: {err}");
        VulkanLoadError
    })?;

    tracing::debug!("vulkan instance created");

    let gpus: Vec<vk::PhysicalDevice> =
        unsafe { instance.enumerate_physical_devices() }.map_err(|err| {
            tracing::error!("no vulkan capable gpus: {err}");
            VulkanLoadError
        })?;

    // find the (prob) 'fastest' gpu
    let (gpu, queue_family): (vk::PhysicalDevice, QueueFamily) = best_gpu(&instance, &gpus)
        .ok_or_else(|| {
            tracing::error!("no suitable gpus");
            VulkanLoadError
        })?;

    let q_info = [vk::DeviceQueueCreateInfo::builder()
        .queue_priorities(&[1.0])
        .queue_family_index(queue_family.0)
        .build()];
    let info = vk::DeviceCreateInfo::builder().queue_create_infos(&q_info);
    let device: ash::Device =
        unsafe { instance.create_device(gpu, &info, None) }.map_err(|err| {
            tracing::error!("could not create a vulkan device: {err}");
            VulkanLoadError
        })?;

    let _queue = unsafe { device.get_device_queue(queue_family.0, 0) };

    todo!("my RX 6900 xt doesn't support VK_KHR_cooperative_matrix after all :( (with RADV)\nhttps://vulkan.gpuinfo.org/listdevicescoverage.php?extension=VK_KHR_cooperative_matrix&platform=linux")

    // Ok(Backend { instance, gpu })
}

//

struct QueueFamily(u32);

fn best_gpu(
    instance: &ash::Instance,
    gpus: &[vk::PhysicalDevice],
) -> Option<(vk::PhysicalDevice, QueueFamily)> {
    gpus.iter()
        .map(|p_dev| {
            let props = unsafe { instance.get_physical_device_properties(*p_dev) };
            (*p_dev, props)
        })
        // filter gpus that support compute shaders
        .filter_map(|(p_dev, props)| {
            let queue_family_index =
                unsafe { instance.get_physical_device_queue_family_properties(p_dev) }
                    .into_iter()
                    .enumerate()
                    // find the first suitable queue family
                    .find(|(_, props)| props.queue_flags.contains(vk::QueueFlags::COMPUTE))
                    // convert the index into a newtype vulkan compatible index
                    .and_then(|(i, _)| Some(QueueFamily(i.try_into().ok()?)));

            if queue_family_index.is_none() {
                let name = as_cstr(&props.device_name[..]).unwrap_or("<invalid gpu name>");
                tracing::info!("gpu `{name}` discarded: no compute shader support");
            }

            Some((p_dev, props, queue_family_index?))
        })
        // filter gpus that can even provide available extensions
        .filter_map(|(p_dev, props, queue)| {
            match unsafe { instance.enumerate_device_extension_properties(p_dev) } {
                Ok(extensions) => Some((p_dev, props, extensions, queue)),
                Err(err) => {
                    let name = as_cstr(&props.device_name[..]).unwrap_or("<invalid gpu name>");
                    tracing::error!(
                        "gpu `{name}` enumerate_device_extension_properties error: {err}"
                    );

                    None
                }
            }
        })
        // filter gpus that support VK_KHR_cooperative_matrix
        .filter(|(_, props, extensions, _)| {
            tracing::info!("extensions: {}", extensions.len());

            let suitable = extensions.iter().any(|extension| {
                tracing::debug!("ext: {:?}", as_cstr(&extension.extension_name[..]));

                as_cstr(&extension.extension_name[..]) == Some("VK_KHR_cooperative_matrix")
            });

            if !suitable {
                let name = as_cstr(&props.device_name[..]).unwrap_or("<invalid gpu name>");
                tracing::info!("gpu `{name}` discarded: no VK_KHR_cooperative_matrix support");
            }

            suitable
        })
        // pick the gpu with the most device local VRAM
        .max_by_key(|(p_dev, _, _, _)| {
            let memory = unsafe { instance.get_physical_device_memory_properties(*p_dev) };

            memory
                .memory_heaps
                .iter()
                .take(memory.memory_heap_count as _)
                .filter(|heap| heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL))
                .map(|heap| heap.size as u128)
                .sum::<u128>()
        })
        // yeet the cached props out of here
        .map(|(p_dev, _, _, queue_family)| (p_dev, queue_family))
}

fn as_cstr(vk_str: &[ffi::c_char]) -> Option<&str> {
    CStr::from_bytes_until_nul(bytemuck::cast_slice(vk_str))
        .ok()
        .and_then(|str| str.to_str().ok())
}

// pub fn instance() -> Result<&'static ash::Instance, VulkanLoadError> {
//     static VK_INSTANCE: OnceLock<Result<ash::Instance, VulkanLoadError>> = OnceLock::new();

//     fn new_instance() -> Result<ash::Instance, VulkanLoadError> {
//         let entry = unsafe { ash::Entry::load() }.map_err(|err| {
//             tracing::warn!("vulkan cannot be loaded: {err}");
//             VulkanLoadError
//         })?;

//         tracing::debug!("vulkan entry loaded");

//         let info = vk::InstanceCreateInfo::builder().build();
//         let inst = unsafe { entry.create_instance(&info, None) }.map_err(|err| {
//             tracing::error!("vulkan instance cannot be created: {err}");
//             VulkanLoadError
//         })?;

//         tracing::debug!("vulkan instance created");

//         Ok(inst)
//     }

//     VK_INSTANCE
//         .get_or_init(new_instance)
//         .as_ref()
//         .map_err(|e| *e)
// }
