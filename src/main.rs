#![allow(dead_code, unused)]
use vulkano::command_buffer::CopyBufferInfo;
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceCreateInfo, Features, QueueCreateInfo};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::sync:: {self, GpuFuture};
use std::time::Instant;

fn main() {
    // Create application's entry to Vulkan API
    let instance = Instance::new(InstanceCreateInfo::default()).unwrap();

    // List all the physical devices that supports Vulkan
    let physical_device = PhysicalDevice::enumerate(&instance).next().unwrap();

    // Find and select the first queue family (threads group) that supports graphics and compute
    let queue_family = physical_device
        .queue_families()
        .find(|&queue_family| queue_family.supports_graphics() && queue_family.supports_compute())
        .unwrap();

    // Create vulkan context from the physical device using the selected queue family.
    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
            ..Default::default()
        },
    )
    .unwrap();

    // Selecting the first queue from the selected queue family
    let queue = queues.next().unwrap();

    // Create a vector that goes from 0 to 63 [0, 1, 2, ..., 63]
    let source_content: Vec<i32> = (0..64).collect();
    // Create a cpu accessible buffer from that vector
    let source = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, source_content).unwrap();
    
    // Create a vector filled with 64 zeroes
    let destination_content: Vec<i32> = (0..64).map(|_| 0).collect();
    // Create a cpu accessible buffer from that vector
    let destination = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, destination_content).unwrap();

    // Create command buffer builder with the vulkan context (device) and the selected queue, just for once
    // after we're done with the command buffer, it will free itself (OneTimeSubmit)
    let mut builder = AutoCommandBufferBuilder::primary(
        device.clone(),
        queue.family(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    // Adding the command to the command buffer (command list)
    builder.copy_buffer(CopyBufferInfo::buffers(source.clone(), destination.clone())).unwrap();

    // Building, finalizing and confirming the command buffer
    let command_buffer = builder.build().unwrap();

    // Creating a future that will upload the command buffer to the device's selected queue
    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush() // upload the command buffer to the device, when ready
        .unwrap();

    println!("Waiting for the GPU to complete the operation");
    let before = Instant::now();
    // Waiting for the future to complete (GPU to sync and upload the command buffer)
    future.wait(None).unwrap();
    let after = Instant::now();
    println!("{:?}", (after-before).as_micros());
    println!("GPU DONE!");

    // requesting read access to the source buffer
    let src_content = source.read().unwrap();
    // requesting read access to the destination buffer
    let destination_content = destination.read().unwrap();

    // compare the source buffer's data to the destination buffer's data to make sure the operation was successful
    assert_eq!(&*src_content, &*destination_content);
}
