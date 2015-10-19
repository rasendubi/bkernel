proc  flash_bkernel { filename } {
    poll
    reset halt
    flash probe 0
    flash write_image erase $filename 0x08000000
    reset
}
