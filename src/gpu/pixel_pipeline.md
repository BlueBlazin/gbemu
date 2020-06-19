# Gameboy Pixel Pipeline

## Mode 3 (Pixel Transfer)

Timings:

```py
if WX == 0xFF:
    # 6 cycles
    run_BO1()
    # 8 * 20 cycles
    repeat_run_B01s()
    return

if 1 <= WX <= 0xA5:
    # 6 cycles
    run_BO1()
    # 8 * (SCX % 8 + WX + 1) cycles
    repeat_run_BO1s()

    fetcher_map = window_nametable
    # 6 cycles
    run_W01()
    # 8 * (166.5 - WX) cycles
    repeat_run_W01s()
    return

if WX == 0:
    if 0 <= SCX % 8 <= 6:
        # 7 cycles
        run_B01B()

        fetcher_map = window_nametable
        # 6 cycles
        run_W01()
        # 8 * (167.5 + SCX % 8) cycles
        repeat_run_W01s()
        return
    if SCX % 8 == 7:
        # 7 cycles
        run_B01B()

        fetcher_map = window_nametable
        # 6 cycles
        run_W01()
        # 8 * (167.5 + 7) cycles + 1 cycle
        # insert 1 extra cycle in first sprite window
        repeat_run_W01s()
        return

if WX == 0xA6:
    # use window nametable
    fetcher_map = window_nametable
    # rendering starts from second tile of each line
    fetcher_x = 1
    # too complicated to list

```
