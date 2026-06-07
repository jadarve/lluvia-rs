
-----------------------------------------------------------
declare class Vec3
    x: number
    y: number
    z: number
end

declare function Vec3(x: number, y: number, z: number): Vec3

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

declare function ComputeNodeDescriptor(): ComputeNodeDescriptor

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
declare class ComputeNode
    push_constants: PushConstants?

    function get_port(self, name: string): any
    function set_constant(self, name: string, value: any): nil
    function get_constant(self, name: string): any
end

-----------------------------------------------------------
declare class Session
    enable_debug: boolean
end


