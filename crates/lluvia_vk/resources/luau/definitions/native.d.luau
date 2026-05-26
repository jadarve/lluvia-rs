
-----------------------------------------------------------
declare class Vec3
    x: number
    y: number
    z: number
end

declare function Vec3(): Vec3

-----------------------------------------------------------
declare class UVec3
    x: number
    y: number
    z: number
end

declare function UVec3(): UVec3

-----------------------------------------------------------

declare class PortDescriptor
    binding: number
    name: string
    direction: number
    port_type: number
end

declare function PortDescriptor(binding: number, name: string, direction: number, port_type: number)

-----------------------------------------------------------
declare class Program
end

declare function load_program(path: string): Program

-----------------------------------------------------------
declare class ComputeNodeDescriptor
    function_name: string
    local_shape: UVec3
    grid_shape: UVec3
    program: Program?

    -- function configure_grid_shape(self, global_shape: UVec3)

    function add_port(self, desc: PortDescriptor): nil
end

declare function ComputeNodeDescriptor(): ComputeNodeDescriptor

-----------------------------------------------------------
declare class ComputeNode

    function getPort(self, name: string): any
    function configureGridShape(self, shape: UVec3): nil
end

-----------------------------------------------------------
declare class Session
    enable_debug: boolean
end


