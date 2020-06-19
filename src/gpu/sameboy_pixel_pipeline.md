# SameBoy Pixel Pipeline

## The First Perspective

### The simplest case - No window, no sprites, no scroll

The bg FIFO will get filled with 8 pixels.

```cpp
/* Fill the FIFO with 8 pixels of "junk", it's going to be dropped anyway. */
fifo_push_bg_row(&gb->bg_fifo, 0, 0, 0, false, false);
```

Position will be `-8` since SCX is 0.

```cpp
gb->position_in_line = - (gb->io_registers[GB_IO_SCX] & 7) - 8;
```

Since `gb->wx_triggered` is always false and there are no sprites, we directly arrive at `abort_fetching_object:`.

Let's recall the current state:

```
bg_fifo->size: 8
position_in_line: -8
fetcher_state: 0
```

```cpp
render_pixel_if_possible(gb);
advance_fetcher_state_machine(gb);
```

In `render_pixel_if_possible`,

```cpp
if (fifo_size(&gb->bg_fifo)) {
    fifo_item = fifo_pop(&gb->bg_fifo);
```

a pixel will get popped off

```cpp
if (gb->position_in_line >= 160 || (gb->disable_rendering && !gb->sgb)) {
    gb->position_in_line++;
    return;
}
```

and position_in_line will increase to `-7`.

In 'advance_fetcher_state_machine' we do `case GB_FETCHER_SLEEP`, and increase fetcher_state to 1.

We continue this for a while as follows:

8 pixels in fifo | GB_FETCHER_SLEEP | fetcher_state: 0 | pos_in_line: -8
7 pixels in fifo | GB_FETCHER_GET_TILE | fetcher_state: 1 | pos_in_line: -7
6 pixels in fifo | GB_FETCHER_SLEEP | fetcher_state: 2 | pos_in_line: -6
5 pixels in fifo | GB_FETCHER_GET_TILE_DATA_LOWER | fetcher_state: 3 | pos_in_line: -5
4 pixels in fifo | GB_FETCHER_SLEEP | fetcher_state: 4 | pos_in_line: -4
3 pixels in fifo | GB_FETCHER_GET_TILE_DATA_HIGH | fetcher_state: 5 | pos_in_line: -3

At this point, we pop off a pixel and advance pos_in_line as always,

2 pixels in fifo | pos_in_line: -2

Also, cycles_for_line: 6.

Now we do `advance_fetcher_state_machine`.

```cpp
case GB_FETCHER_GET_TILE_DATA_HIGH: {
    // ...
}

// fallthrough
case GB_FETCHER_PUSH: {
    if (gb->fetcher_state == 6) {
        /* The background map index increase at this specific point. If this state is not reached,
            it will simply not increase. */
        gb->fetcher_x++;
        gb->fetcher_x &= 0x1f;
    }
    if (gb->fetcher_state < 7) {
        gb->fetcher_state++;
    }
    if (fifo_size(&gb->bg_fifo) > 0) break;

    fifo_push_bg_row(&gb->bg_fifo, gb->current_tile_data[0], gb->current_tile_data[1],
                        gb->current_tile_attributes & 7, gb->current_tile_attributes & 0x80, gb->current_tile_attributes & 0x20);
    gb->fetcher_state = 0;
}
break;
```

Already something different. After doing `case GB_FETCHER_GET_TILE_DATA_HIGH`, we don't break. Rather we continue to the next case `GB_FETCHER_PUSH`.

```cpp
if (gb->fetcher_state < 7) {
    gb->fetcher_state++;
}
if (fifo_size(&gb->bg_fifo) > 0) break;
```

We increment `fetcher_state` and `break` here instead.

2 pixels in fifo | GB_FETCHER_PUSH | fetcher_state: 6 | pos_in_line: -2

Again, we pop a pixel off to enter:

1 pixels in fifo | GB_FETCHER_PUSH | fetcher_state: 6 | pos_in_line: -1

In `advance_fetcher_state_machine`,

```cpp
if (gb->fetcher_state == 6) {
    /* The background map index increase at this specific point. If this state is not reached,
        it will simply not increase. */
    gb->fetcher_x++;
    gb->fetcher_x &= 0x1f;
}

if (fifo_size(&gb->bg_fifo) > 0) break;
```

The fifo is still not empty, so we do break at this point. But we do increment fetcher_x.

1 pixels in fifo | GB_FETCHER_PUSH | fetcher_state: 7 | pos_in_line: -1

Finally, the next `render_pixel_if_possible` gets us to

0 pixels in fifo | GB_FETCHER_PUSH | fetcher_state: 7 | pos_in_line: 0

In `advance_fetcher_state_machine`,

```cpp
case GB_FETCHER_PUSH: {
    // ...
    fifo_push_bg_row(&gb->bg_fifo, gb->current_tile_data[0], gb->current_tile_data[1],
                        gb->current_tile_attributes & 7, gb->current_tile_attributes & 0x80, gb->current_tile_attributes & 0x20);
    gb->fetcher_state = 0;
}
break;
```

`fifo_size` is no longer greater than 0, so we push the first tile row and cicle back to fetcher_state 0. All of the initial "junk" pixels have been popped off the fifo.

```
cycles_for_line: 8
bg_fetcher->size: 8
```

### No window, no sprites, SCX > 0

In this case, everything that happened before is identical. The only difference is at the end of those first 8 junk pixels `pos_in_line` is still not 0.

Since

```cpp
gb->position_in_line = - (gb->io_registers[GB_IO_SCX] & 7) - 8;
```

and SCX > 0, we will have `gb->position_in_line < 0`. So we continue to throw away pixels from the first tile row until `gb->position_in_line` reaches 0. That's it, nothing more to it.

### No sprites, window with 0 < WX < 166

Now let's look at what happens when we have a window. In particular, we'll look at the simplest case where WX is between 0 and 166.

```cpp
if (!gb->wx_triggered && gb->wy_triggered && (gb->io_registers[GB_IO_LCDC] & 0x20)) {
    bool should_activate_window = false;

    else if (gb->io_registers[GB_IO_WX] < 166 + GB_is_cgb(gb)) {
        if (gb->io_registers[GB_IO_WX] == (uint8_t) (gb->position_in_line + 7)) {
            should_activate_window = true;
        }
    }

    if (should_activate_window) {
        gb->window_y++;
        gb->wx_triggered = true;
        gb->window_tile_x = 0;
        fifo_clear(&gb->bg_fifo);
        gb->fetcher_state = 0;
        gb->window_is_being_fetched = true;
    }
}
```

`window_y` which is initialized to `-1` gets incremented. The `wx_triggered` flag gets set, `window_tile_x` is set to 0, the fifo is cleared, the `fetcher_state` is reset to 0 and the `window_is_being_fetched` flag gets set.

Nothing interesting happens until `GB_FETCHER_GET_TILE_DATA_HIGH`.

```cpp
case GB_FETCHER_GET_TILE_DATA_HIGH: {
    // ...
    if (gb->wx_triggered) {
        gb->window_tile_x++;
        gb->window_tile_x &= 0x1f;
    }
}

// fallthrough
case GB_FETCHER_PUSH: {
    if (gb->fetcher_state == 6) {
        /* The background map index increase at this specific point. If this state is not reached,
            it will simply not increase. */
        gb->fetcher_x++;
        gb->fetcher_x &= 0x1f;
    }
    if (gb->fetcher_state < 7) {
        gb->fetcher_state++;
    }
    if (fifo_size(&gb->bg_fifo) > 0) break;

    fifo_push_bg_row(&gb->bg_fifo, gb->current_tile_data[0], gb->current_tile_data[1],
                        gb->current_tile_attributes & 7, gb->current_tile_attributes & 0x80, gb->current_tile_attributes & 0x20);
    gb->fetcher_state = 0;
}
break;
```

First, in `GB_FETCHER_GET_TILE_DATA_HIGH` we increment `window_tile_x` from 0 to 1. But more interestingly, in `GB_FETCHER_PUSH`, `fifo_size(&gb->bg_fifo) > 0` is **not** true.

If you've forgotten, this condition **was** true when we fetched our first bg tile row (due to the 8 junk pixels).

So on the 6th cycle we push the window tile row. The next time around, `bg_fifo->size` won't be 0 and the fetching of window tile rows will continue at the normal 8 clocks per tile row rate.

### No sprites, window = 0

TODO

### No sprites, window = 166

TODO

### Sprites

TODO
