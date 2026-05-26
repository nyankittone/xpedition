let SessionLoad = 1
let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1
let v:this_session=expand("<sfile>:p")
doautoall SessionLoadPre
silent only
silent tabonly
cd ~/Projects/Programming/xpedition
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
let s:shortmess_save = &shortmess
set shortmess+=aoO
badd +396 src/render.rs
badd +113 src/events.rs
badd +59 src/main.rs
badd +71 src/render_entities.rs
badd +25 src/entities.rs
badd +1 src/render_pipeline.rs
badd +362 src/tick.rs
badd +9057 term://~/Projects/Programming/xpedition//33189:/usr/bin/bash
badd +23 term://~/Projects/Programming/xpedition//69100:/usr/bin/bash
badd +1 src/physics.rs
badd +9 src/lib.rs
badd +103 src/math_types.rs
argglobal
%argdel
$argadd src/render.rs
set stal=2
tabnew +setlocal\ bufhidden=wipe
tabnew +setlocal\ bufhidden=wipe
tabnew +setlocal\ bufhidden=wipe
tabrewind
edit src/render.rs
let s:save_splitbelow = &splitbelow
let s:save_splitright = &splitright
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
wincmd _ | wincmd |
split
1wincmd k
wincmd w
let &splitbelow = s:save_splitbelow
let &splitright = s:save_splitright
wincmd t
let s:save_winminheight = &winminheight
let s:save_winminwidth = &winminwidth
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 118 + 118) / 237)
exe '2resize ' . ((&lines * 44 + 29) / 59)
exe 'vert 2resize ' . ((&columns * 118 + 118) / 237)
exe '3resize ' . ((&lines * 11 + 29) / 59)
exe 'vert 3resize ' . ((&columns * 118 + 118) / 237)
argglobal
balt src/main.rs
setlocal foldmethod=manual
setlocal foldexpr=0
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldenable
silent! normal! zE
let &fdl = &fdl
let s:l = 31 - ((30 * winheight(0) + 28) / 56)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 31
normal! 0
wincmd w
argglobal
if bufexists(fnamemodify("term://~/Projects/Programming/xpedition//33189:/usr/bin/bash", ":p")) | buffer term://~/Projects/Programming/xpedition//33189:/usr/bin/bash | else | edit term://~/Projects/Programming/xpedition//33189:/usr/bin/bash | endif
if &buftype ==# 'terminal'
  silent file term://~/Projects/Programming/xpedition//33189:/usr/bin/bash
endif
balt src/render.rs
setlocal foldmethod=manual
setlocal foldexpr=0
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldenable
let s:l = 9057 - ((43 * winheight(0) + 22) / 44)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 9057
normal! 0
lcd ~/Projects/Programming/xpedition
wincmd w
argglobal
if bufexists(fnamemodify("~/Projects/Programming/xpedition/src/render_pipeline.rs", ":p")) | buffer ~/Projects/Programming/xpedition/src/render_pipeline.rs | else | edit ~/Projects/Programming/xpedition/src/render_pipeline.rs | endif
if &buftype ==# 'terminal'
  silent file ~/Projects/Programming/xpedition/src/render_pipeline.rs
endif
balt term://~/Projects/Programming/xpedition//33189:/usr/bin/bash
setlocal foldmethod=manual
setlocal foldexpr=0
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldenable
silent! normal! zE
let &fdl = &fdl
let s:l = 2 - ((1 * winheight(0) + 5) / 11)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 2
normal! 0
lcd ~/Projects/Programming/xpedition
wincmd w
exe 'vert 1resize ' . ((&columns * 118 + 118) / 237)
exe '2resize ' . ((&lines * 44 + 29) / 59)
exe 'vert 2resize ' . ((&columns * 118 + 118) / 237)
exe '3resize ' . ((&lines * 11 + 29) / 59)
exe 'vert 3resize ' . ((&columns * 118 + 118) / 237)
tabnext
edit ~/Projects/Programming/xpedition/src/tick.rs
let s:save_splitbelow = &splitbelow
let s:save_splitright = &splitright
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
wincmd _ | wincmd |
split
1wincmd k
wincmd w
let &splitbelow = s:save_splitbelow
let &splitright = s:save_splitright
wincmd t
let s:save_winminheight = &winminheight
let s:save_winminwidth = &winminwidth
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 118 + 118) / 237)
exe '2resize ' . ((&lines * 39 + 29) / 59)
exe 'vert 2resize ' . ((&columns * 118 + 118) / 237)
exe '3resize ' . ((&lines * 16 + 29) / 59)
exe 'vert 3resize ' . ((&columns * 118 + 118) / 237)
argglobal
balt ~/Projects/Programming/xpedition/src/entities.rs
setlocal foldmethod=manual
setlocal foldexpr=0
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldenable
silent! normal! zE
let &fdl = &fdl
let s:l = 376 - ((42 * winheight(0) + 28) / 56)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 376
normal! 010|
lcd ~/Projects/Programming/xpedition
wincmd w
argglobal
if bufexists(fnamemodify("~/Projects/Programming/xpedition/src/entities.rs", ":p")) | buffer ~/Projects/Programming/xpedition/src/entities.rs | else | edit ~/Projects/Programming/xpedition/src/entities.rs | endif
if &buftype ==# 'terminal'
  silent file ~/Projects/Programming/xpedition/src/entities.rs
endif
setlocal foldmethod=manual
setlocal foldexpr=0
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldenable
silent! normal! zE
let &fdl = &fdl
let s:l = 25 - ((24 * winheight(0) + 19) / 39)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 25
normal! 034|
lcd ~/Projects/Programming/xpedition
wincmd w
argglobal
if bufexists(fnamemodify("term://~/Projects/Programming/xpedition//33189:/usr/bin/bash", ":p")) | buffer term://~/Projects/Programming/xpedition//33189:/usr/bin/bash | else | edit term://~/Projects/Programming/xpedition//33189:/usr/bin/bash | endif
if &buftype ==# 'terminal'
  silent file term://~/Projects/Programming/xpedition//33189:/usr/bin/bash
endif
balt ~/Projects/Programming/xpedition/src/events.rs
setlocal foldmethod=manual
setlocal foldexpr=0
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldenable
let s:l = 9057 - ((15 * winheight(0) + 8) / 16)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 9057
normal! 03|
lcd ~/Projects/Programming/xpedition
wincmd w
exe 'vert 1resize ' . ((&columns * 118 + 118) / 237)
exe '2resize ' . ((&lines * 39 + 29) / 59)
exe 'vert 2resize ' . ((&columns * 118 + 118) / 237)
exe '3resize ' . ((&lines * 16 + 29) / 59)
exe 'vert 3resize ' . ((&columns * 118 + 118) / 237)
tabnext
edit ~/Projects/Programming/xpedition/src/math_types.rs
let s:save_splitbelow = &splitbelow
let s:save_splitright = &splitright
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
let &splitbelow = s:save_splitbelow
let &splitright = s:save_splitright
wincmd t
let s:save_winminheight = &winminheight
let s:save_winminwidth = &winminwidth
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 118 + 118) / 237)
exe 'vert 2resize ' . ((&columns * 118 + 118) / 237)
argglobal
balt ~/Projects/Programming/xpedition/src/lib.rs
setlocal foldmethod=manual
setlocal foldexpr=0
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldenable
silent! normal! zE
let &fdl = &fdl
let s:l = 103 - ((18 * winheight(0) + 28) / 56)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 103
normal! 05|
lcd ~/Projects/Programming/xpedition
wincmd w
argglobal
if bufexists(fnamemodify("term://~/Projects/Programming/xpedition//33189:/usr/bin/bash", ":p")) | buffer term://~/Projects/Programming/xpedition//33189:/usr/bin/bash | else | edit term://~/Projects/Programming/xpedition//33189:/usr/bin/bash | endif
if &buftype ==# 'terminal'
  silent file term://~/Projects/Programming/xpedition//33189:/usr/bin/bash
endif
setlocal foldmethod=manual
setlocal foldexpr=0
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldenable
let s:l = 9057 - ((55 * winheight(0) + 28) / 56)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 9057
normal! 0
lcd ~/Projects/Programming/xpedition
wincmd w
exe 'vert 1resize ' . ((&columns * 118 + 118) / 237)
exe 'vert 2resize ' . ((&columns * 118 + 118) / 237)
tabnext
edit ~/Projects/Programming/xpedition/src/lib.rs
let s:save_splitbelow = &splitbelow
let s:save_splitright = &splitright
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
let &splitbelow = s:save_splitbelow
let &splitright = s:save_splitright
wincmd t
let s:save_winminheight = &winminheight
let s:save_winminwidth = &winminwidth
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 118 + 118) / 237)
exe 'vert 2resize ' . ((&columns * 118 + 118) / 237)
argglobal
balt ~/Projects/Programming/xpedition/src/main.rs
setlocal foldmethod=manual
setlocal foldexpr=0
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldenable
silent! normal! zE
let &fdl = &fdl
let s:l = 9 - ((8 * winheight(0) + 28) / 56)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 9
normal! 015|
lcd ~/Projects/Programming/xpedition
wincmd w
argglobal
if bufexists(fnamemodify("term://~/Projects/Programming/xpedition//33189:/usr/bin/bash", ":p")) | buffer term://~/Projects/Programming/xpedition//33189:/usr/bin/bash | else | edit term://~/Projects/Programming/xpedition//33189:/usr/bin/bash | endif
if &buftype ==# 'terminal'
  silent file term://~/Projects/Programming/xpedition//33189:/usr/bin/bash
endif
balt ~/Projects/Programming/xpedition/src/main.rs
setlocal foldmethod=manual
setlocal foldexpr=0
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldenable
let s:l = 1 - ((0 * winheight(0) + 28) / 56)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 1
normal! 0
lcd ~/Projects/Programming/xpedition
wincmd w
exe 'vert 1resize ' . ((&columns * 118 + 118) / 237)
exe 'vert 2resize ' . ((&columns * 118 + 118) / 237)
tabnext 2
set stal=1
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20
let &shortmess = s:shortmess_save
let &winminheight = s:save_winminheight
let &winminwidth = s:save_winminwidth
let s:sx = expand("<sfile>:p:r")."x.vim"
if filereadable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &g:so = s:so_save | let &g:siso = s:siso_save
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :
