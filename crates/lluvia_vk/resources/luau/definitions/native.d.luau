
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
    local_shape: UVec3
    grid_shape: UVec3
    program: Program?

    function add_port(self, desc: PortDescriptor): nil
    function set_constant(self, name: string, value: any): nil
    function get_constant(self, name: string): any
end

declare function ComputeNodeDescriptor(): ComputeNodeDescriptor

-----------------------------------------------------------
declare class ComputeNode
    push_constants: PushConstants?

    function get_port(self, name: string): any
    function configure_grid_shape(self, shape: UVec3): nil
    function set_constant(self, name: string, value: any): nil
    function get_constant(self, name: string): any
end

-----------------------------------------------------------
declare class Session
    enable_debug: boolean
end


