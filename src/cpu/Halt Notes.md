The `HALT` instruction takes 4 cycles.

```cpp
static void halt(GB_gameboy_t *gb, uint8_t opcode)
{
    assert(gb->pending_cycles == 4);
    gb->pending_cycles = 0;
    GB_advance_cycles(gb, 4);

    gb->halted = true;
    /* Despite what some online documentations say, the HALT bug also happens on a CGB, in both CGB and DMG modes. */
    if (((gb->interrupt_enable & gb->io_registers[GB_IO_IF] & 0x1F) != 0)) {
        if (gb->ime) {
            gb->halted = false;
            gb->pc--;
        }
        else {
            gb->halted = false;
            gb->halt_bug = true;
        }
    }
    gb->just_halted = true;
}
```

## HALT Mode

During halt mode the CPU does not fetch and execute new instructions. Halt mode is exited when the interrupt line becomes nonzero (IE & IF & 0x1F != 0). When exiting HALT, if IME is set, then the interrupt is also handled whereas if it's not set then the CPU simply exits HALT mode and resumes executing instructions as usual.

## HALT Instruction

The halt instruction is used to put the CPU in halt mode. The only reason to even write about it is because it has 3 different behaviors depending on IE, IF, and IME.

1. IME = 1: This is the nominal case. CPU enters HALT mode. The behavior in HALT mode is detailed above.

2. IME = 0:
   - IE & IF & 0x1F == 0: As before, CPU enters HALT mode. Nothing abnormal here.
   - IE & IF & 0x1F != 0: This results in a bug known as the HALT bug. HALT mode isn't entered, but rather the CPU fails to increase PC after the next instruction.
