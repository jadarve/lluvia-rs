
-----------------------------------------------------------
declare class Vec3
    x: number
    y: number
    z: number
end

declare function Vec3(x: number, y: number, z: number): Vec3

-----------------------------------------------------------
declare class UVec2
    x: number
    y: number
end

declare function UVec2(x: number, y: number): UVec2

-----------------------------------------------------------
declare class UVec3
    x: number
    y: number
    z: number
end

declare function UVec3(x: number, y: number, z: number): UVec3

-----------------------------------------------------------

declare class PortDescriptor
    binding: number
    name: string
    direction: number
    port_type: number
end

declare function PortDescriptor(binding: number, name: string, direction: number, port_type: number): PortDescriptor

-----------------------------------------------------------
declare class Program
end

declare function load_program(path: string): Program

-----------------------------------------------------------
declare class Buffer
    function size(self): number
    function map_and_set_from_vector_uint8(self, data: VectorUint8): nil
end

-----------------------------------------------------------
declare class Constant
	function set(self, value: any): nil
	function get(self): any
end

declare function Constant(): Constant

-----------------------------------------------------------
declare class PushConstants
	function push_f32(self, value: number): nil
	function push_i32(self, value: number): nil
end

declare function PushConstants(): PushConstants

-----------------------------------------------------------
declare class ComputeNodeDescriptor
    function_name: string
    global_shape: UVec3
    program: Program?

    function builder(self): ComputeNodeDescriptorBuilder
    function add_port(self, desc: PortDescriptor): nil
    function set_constant(self, name: string, value: any): nil
    function get_constant(self, name: string): any
end

-----------------------------------------------------------
declare class ComputeNodeDescriptorBuilder
    function function_name(self, name: string): ComputeNodeDescriptorBuilder
    function global_shape(self, shape: UVec3): ComputeNodeDescriptorBuilder
    function program(self, program: Program?): ComputeNodeDescriptorBuilder
    function add_port(self, desc: PortDescriptor): ComputeNodeDescriptorBuilder
    function add_constant(self, name: string, value: any): ComputeNodeDescriptorBuilder
    function build(self): ComputeNodeDescriptor
end

declare function ComputeNodeDescriptorBuilder(): ComputeNodeDescriptorBuilder


-----------------------------------------------------------
declare class ImageView
    width: number
    height: number
    depth: number
    channel_count: number
end

-----------------------------------------------------------
declare class ComputeNode
    push_constants: PushConstants?

    function get_port(self, name: string): any
    function set_constant(self, name: string, value: any): nil
    function get_constant(self, name: string): any
    function bind(self, name: string, obj: Buffer | ImageView): nil
    function init(self): nil
end

-----------------------------------------------------------
declare class SessionDescriptor
    enable_debug: boolean
end

declare class Session
    function load_program(self, path: string): Program
end

-----------------------------------------------------------
declare class ContainerNodeDescriptor
    builder_name: string

    function get_constant(self, name: string): any
end

-----------------------------------------------------------
declare class ContainerNodeDescriptorBuilder
    function builder_name(self, name: string): ContainerNodeDescriptorBuilder
    function add_port(self, desc: PortDescriptor): ContainerNodeDescriptorBuilder
    function add_constant(self, name: string, value: any): ContainerNodeDescriptorBuilder
    function build(self): ContainerNodeDescriptor
end

declare function ContainerNodeDescriptorBuilder(): ContainerNodeDescriptorBuilder

-----------------------------------------------------------
declare class ContainerNode
    function get_constant(self, name: string): any
    function set_constant(self, name: string, value: any): nil
    function get_constants(self): { [string]: any }
    function get_port(self, name: string): any
    function bind(self, name: string, obj: Buffer | ImageView): nil
    function add_node(self, name: string, child: ComputeNode | ContainerNode): nil
    function get_node(self, name: string): ComputeNode | ContainerNode | nil
    function init(self): nil
end

declare function create_compute_node(builder_name: string, args: Arguments?): ComputeNode
declare function create_container_node(builder_name: string, args: Arguments?): ContainerNode
declare function create_image_view(width: number, height: number, depth: number, channel_count: number, channel_type: string): ImageView

-----------------------------------------------------------
declare class CommandBuffer
    function run(self, node: ComputeNode | ContainerNode): nil
end

-----------------------------------------------------------
declare class VectorUint8
    function size(self): number
end

declare class Image
    function create_image_view(self, desc: ImageViewDescriptor): ImageView
end

declare class ImageDescriptor
end

declare class ImageViewDescriptor
end

declare class CommandBufferBuilder
    function copy_buffer_to_image(self, src: Buffer, dst: Image): nil
    function build(self): CommandBuffer
end

declare function from_base64(encoded: string): VectorUint8
declare function create_buffer(size: number, host_visible: boolean): Buffer
declare function create_image(desc: ImageDescriptor): Image
declare function create_command_buffer_builder(): CommandBufferBuilder
declare function run_command_buffer(cmd_buf: CommandBuffer): nil
declare function create_image_descriptor(depth: number, height: number, width: number, channel_count: number, channel_type: string): ImageDescriptor
declare function create_image_view_descriptor(address_mode: number, filter_mode: number, normalized_coords: boolean, is_sampled: boolean): ImageViewDescriptor

-----------------------------------------------------------
export type Argument = number | boolean | Vec3 | UVec3 | UVec2 | nil
export type Arguments = { [string]: Argument }
