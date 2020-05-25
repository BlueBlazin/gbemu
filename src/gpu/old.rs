// fn fifo_tick2(&mut self) {
//     if self.bg_fetcher.fetching == FetchType::Background
//         && self.is_win_enabled()
//         && self.is_win_pixel()
//     {
//         self.bg_fifo.clear_fifo();
//         self.bg_fetcher.reset();
//         self.bg_fetcher.fetching = FetchType::Window;
//         self.bg_fifo.winx = (self.position.lx + 7 - self.position.window_x) % 8;
//     }

//     if self.bg_fifo.size() <= 8 {
//         return;
//     }

//     match self.bg_fetcher.fetching {
//         FetchType::Background if self.bg_fifo.scx > 0 => {
//             self.bg_fifo.pop();
//             self.bg_fifo.scx -= 1;
//         }
//         FetchType::Window if self.bg_fifo.winx > 0 => {
//             self.bg_fifo.pop();
//             self.bg_fifo.winx -= 1;
//         }
//         _ if self.bg_fifo.objx > 0 => {
//             self.bg_fifo.objx -= 1;
//         }
//         _ => {
//             let item = self.bg_fifo.pop();
//             let (r, g, b) = if self.lcdc.lcdc0 == 0 {
//                 self.get_rgb(0, item.palette)
//             } else {
//                 self.get_rgb(item.value, item.palette)
//             };
//             self.update_screen_row(self.position.lx as usize, r, g, b);

//             self.position.lx += 1;
//             self.check_sprite_comparators();
//         }
//     }
// }
