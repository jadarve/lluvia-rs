import pytest
import lluvia_vk as ll

def test_create_session():
    # TODO: builder to create the descriptor
    session = ll.Session(True)

    dev_buffer = session.create_buffer_device_local(1024)
    assert dev_buffer.size == 1024

    host_buffer = session.create_buffer_host_visible(512)
    assert host_buffer.size == 512

    data = bytes([1] * 512)
    host_buffer.write(data)
    read_data = host_buffer.read()
    
    assert list(data) == read_data

    # TODO: explose API to tell buffer is host visible.

    # usage_flags = buffer.usage()
    # assert usage_flags == ...

    # session = ll.Session()
    # assert session.is_valid()


if __name__ == "__main__":
    raise SystemExit(pytest.main([__file__]))
