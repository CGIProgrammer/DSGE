use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator,
        AutoCommandBufferBuilder,
        CommandBufferUsage,
        CopyBufferInfoTyped, // PrimaryCommandBuffer,
        PrimaryAutoCommandBuffer,
        PrimaryCommandBufferAbstract,
    },
    device::Queue,
    memory::{allocator::{
        AllocationCreateInfo, GenericMemoryAllocator, MemoryTypeFilter, StandardMemoryAllocator, Suballocator
    }, MemoryPropertyFlags},
    sync::GpuFuture,
};

pub struct PrimaryCommandBufferAssembler {
    command_buffer_builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    //allocator: Arc<StandardCommandBufferAllocator>,
    queue: Arc<Queue>,
}

impl Deref for PrimaryCommandBufferAssembler {
    type Target = AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>;

    fn deref(&self) -> &Self::Target {
        &self.command_buffer_builder
    }
}

impl DerefMut for PrimaryCommandBufferAssembler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.command_buffer_builder
    }
}

impl PrimaryCommandBufferAssembler {
    pub fn execute_after(
        self,
        future: Option<Box<dyn GpuFuture>>,
    ) -> Result<Box<dyn GpuFuture>, String> {
        let future = future.unwrap_or(vulkano::sync::now(self.queue.device().clone()).boxed());
        let future = match self.command_buffer_builder.build() {
            Ok(cb) => match cb.execute_after(future, self.queue.clone()) {
                Ok(future) => future.boxed(),
                Err(err) => return Err(err.to_string()),
            },
            Err(err) => return Err(err.to_string()),
        };
        Ok(future)
    }

    pub fn build_buffer(self) -> Arc<PrimaryAutoCommandBuffer> {
        self.command_buffer_builder.build().unwrap()
    }
}

pub struct CommandBufferFather {
    queue: Arc<Queue>,
    allocator: Arc<StandardCommandBufferAllocator>,
}

impl CommandBufferFather {
    pub fn new(queue: Arc<Queue>) -> Self {
        Self {
            queue: queue.clone(),
            allocator: Arc::new(StandardCommandBufferAllocator::new(
                queue.device().clone(),
                Default::default(),
            )),
        }
    }

    #[inline(always)]
    pub fn new_primary(&self) -> Result<PrimaryCommandBufferAssembler, String> {
        let pcbb = AutoCommandBufferBuilder::primary(
            self.allocator.as_ref(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        );
        match pcbb {
            Ok(pcbb) => Ok(PrimaryCommandBufferAssembler {
                //allocator: self.allocator.clone(),
                queue: self.queue.clone(),
                command_buffer_builder: pcbb,
            }),
            Err(err) => Err(err.to_string()),
        }
    }

    #[inline(always)]
    pub fn execute_in_new_primary<F, T>(
        &self,
        future: Option<Box<dyn GpuFuture>>,
        f: F,
    ) -> Result<(T, Box<dyn GpuFuture>), String>
    where
        F: FnOnce(&mut PrimaryCommandBufferAssembler) -> T,
    {
        let mut command_buffer_builder = self.new_primary()?;
        let result = f(&mut command_buffer_builder);
        Ok((result, command_buffer_builder.execute_after(future)?))
    }

    #[inline(always)]
    pub fn new_primary_instant<F, T>(&self, f: F) -> Result<(T, Arc<PrimaryAutoCommandBuffer>), String>
    where
        F: FnOnce(&mut PrimaryCommandBufferAssembler) -> T,
    {
        let mut command_buffer_builder = self.new_primary()?;
        let result = f(&mut command_buffer_builder);
        let command_buffer = command_buffer_builder.build_buffer();
        Ok((result, command_buffer))
    }

    #[inline(always)]
    pub fn allocator(&self) -> &Arc<StandardCommandBufferAllocator> {
        &self.allocator
    }

    #[inline(always)]
    pub fn queue(&self) -> &Arc<Queue> {
        &self.queue
    }

    /*#[inline(always)]
    pub fn new_primary_command_buffer(
        &self,
    ) -> Result<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, CommandBufferBeginError> {
        AutoCommandBufferBuilder::primary(
            &self.allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
    }

    #[inline(always)]
    pub fn build_primary_command_buffer_builder(
        command_buffer_builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) -> Result<PrimaryAutoCommandBuffer, String> {
        match command_buffer_builder.build() {
            Ok(ok) => Ok(ok),
            Err(err) => return Err(format!("{err:?}")),
        }
    }

    #[inline(always)]
    pub fn execute_primary_command_buffer_builder(
        &self,
        command_buffer_builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        future: Option<Box<dyn GpuFuture>>,
    ) -> Result<Box<dyn GpuFuture>, String> {
        let device = self.queue.device().clone();
        let future = future.unwrap_or(vulkano::sync::now(device.clone()).boxed());
        match match command_buffer_builder.build() {
            Ok(ok) => ok,
            Err(err) => return Err(format!("Ошибка сборки буфера команд: {err:?}")),
        }
        .execute_after(future, self.queue.clone())
        {
            Ok(ok) => Ok(ok.boxed()),
            Err(err) => Err(format!("Ошибка выполнения буфера команд: {err:?}")),
        }
    }

    /// Декоратор для вызовов команд с созданием нового буфера команд.
    #[inline(always)]
    pub fn execute_in_new_primary_command_buffer<F>(
        &self,
        future: Option<Box<dyn GpuFuture>>,
        f: F,
    ) -> Result<Box<dyn GpuFuture>, String>
    where
        F: FnOnce(&mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>),
    {
        let mut command_buffer_builder = self.new_primary_command_buffer().unwrap();
        f(&mut command_buffer_builder);
        self.execute_primary_command_buffer_builder(command_buffer_builder, future)
    }

    /// Декоратор для вызовов команд с созданием нового буфера команд и последующим его выполнением.
    #[inline(always)]
    pub fn add_to_new_primary_command_buffer<F>(
        &self,
        f: F,
    ) -> Result<PrimaryAutoCommandBuffer, String>
    where
        F: FnOnce(&mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>),
    {
        let mut command_buffer_builder = self.new_primary_command_buffer().unwrap();
        f(&mut command_buffer_builder);
        Self::build_primary_command_buffer_builder(command_buffer_builder)
    }*/
}

pub fn new_cpu_buffer_from_iter<T, I, A>(
    usage: BufferUsage,
    allocator: Arc<GenericMemoryAllocator<A>>,
    iter: I,
) -> Result<Subbuffer<[T]>, String>
where
    T: BufferContents,
    A: Suballocator + std::marker::Send + 'static,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator
{
    let buffer = Buffer::from_iter(
        allocator,
        BufferCreateInfo {
            usage: usage,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        iter,
    );
    /*if let Ok(buffer) = &buffer {
        println!("Создан новый буфер CPU размером {} байт", buffer.size());
    }*/
    buffer.map_err(|err| err.to_string())
    // match buffer {
    //     Ok(buffer) => Ok(buffer),
    //     Err(err) => Err(err.to_string())
    // }
}

pub trait CommandBufferShortcuts {
    
    fn move_buffer_to_device<T, A>(
        &mut self,
        subbuffer: Subbuffer<[T]>,
        allocator: Arc<GenericMemoryAllocator<A>>,
    ) -> Result<Subbuffer<[T]>, String>
    where
        T: BufferContents,
        A: Suballocator + Send + 'static;
    fn new_buffer_on_device_from_iter<T, I, A>(
        &mut self,
        usage: BufferUsage,
        allocator: Arc<GenericMemoryAllocator<A>>,
        iter: I,
    ) -> Result<Subbuffer<[T]>, String>
    where
        T: BufferContents,
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
        A: Suballocator + Send + 'static;
}

impl CommandBufferShortcuts for PrimaryCommandBufferAssembler {
    fn move_buffer_to_device<T, A>(
        &mut self,
        subbuffer: Subbuffer<[T]>,
        allocator: Arc<GenericMemoryAllocator<A>>,
    ) -> Result<Subbuffer<[T]>, String>
    where
        A: Suballocator + Send + 'static,
        T: BufferContents,
    {
        let mut usage = subbuffer.buffer().usage() | BufferUsage::TRANSFER_DST;
        if usage.intersects(BufferUsage::TRANSFER_SRC) {
            usage = usage.symmetric_difference(BufferUsage::TRANSFER_SRC);
        }
        let device_local_buffer = Buffer::new_slice::<T>(
            allocator,
            BufferCreateInfo {
                usage,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter {
                    required_flags: MemoryPropertyFlags::DEVICE_LOCAL,
                    not_preferred_flags: 
                        MemoryPropertyFlags::DEVICE_COHERENT | 
                        MemoryPropertyFlags::HOST_CACHED | 
                        MemoryPropertyFlags::HOST_COHERENT | 
                        MemoryPropertyFlags::DEVICE_COHERENT,
                    ..Default::default()
                },
                ..Default::default()
            },
            subbuffer.len(),
        );
        let device_local_buffer = match device_local_buffer {
            Ok(dlb) => dlb,
            Err(err) => return Err(err.to_string())
        };
        self.copy_buffer(CopyBufferInfoTyped::buffers(
            subbuffer,
            device_local_buffer.clone(),
        ))
        .unwrap();
        Ok(device_local_buffer)
    }

    fn new_buffer_on_device_from_iter<T, I, A>(
        &mut self,
        usage: BufferUsage,
        allocator: Arc<GenericMemoryAllocator<A>>,
        iter: I,
    ) -> Result<Subbuffer<[T]>, String>
    where
        T: BufferContents,
        A: Suballocator + Send + 'static,
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let cpu_buffer = Buffer::from_iter(
            allocator.clone(),
            BufferCreateInfo {
                usage: usage | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE | MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            iter,
        );
        let cpu_buffer = match cpu_buffer {
            Ok(cpuab) => cpuab,
            Err(err) => return Err(err.to_string())
        };
        if usage.intersects(BufferUsage::VERTEX_BUFFER) {
            println!("Создан новый вершинный буфер размером {} байт", cpu_buffer.size());
        }
        if usage.intersects(BufferUsage::INDEX_BUFFER) {
            println!("Создан новый индексный буфер размером {} байт", cpu_buffer.size());
        }
        //Ok(cpu_buffer)
        self.move_buffer_to_device(cpu_buffer, allocator.clone())
    }
}
