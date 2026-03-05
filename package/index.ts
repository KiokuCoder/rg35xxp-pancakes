import {register} from "../scripts/pkg.ts";
import {RetroArch, RetroArchAssets} from "./RetroArch.ts";
import {SDL2, SDL2ALL} from "./sdl2.ts";
import {t507_gpu_drivers} from "./t507_gpu_drivers.ts";
import {gamepad} from "./gamepad.ts";
import {eudev} from "./eudev.ts";
import {busybox} from "./busybox.ts";
import {zlib} from "./zlib.ts";
import {bzip2} from "./bzip2.ts";
import {evtest} from "./evtest.ts";
import {openssh} from "./openssh.ts";
import {openssl} from "./openssl.ts";
import {rsync} from "./rsync.ts";
import {libnl} from "./libnl.ts";
import {connman} from "./connman.ts";
import {iptables} from "./iptables.ts";
import {libnftnl} from "./libnftnl.ts";
import {gnutls} from "./gnutls.ts";
import {nettle} from "./nettle.ts";
import {gmp} from "./gmp.ts";
import {readline} from "./readline.ts";
import {ncurses} from "./ncurses.ts";
import {ffmpeg} from "./ffmpeg.ts";
import {x264} from "./x264.ts";
import {x265} from "./x265.ts";
import {fontconfig} from "./fontconfig.ts";
import {notoCJK} from "./noto-cjk.ts";
import {libretroMgba as mGCore} from "./libretro-mgba.ts";
import {sqlite3} from "./sqlite3.ts";
import {libdrm} from "./libdrm.ts";
import {dbus} from "./dbus.ts";
import {alsa} from "./alsa.ts";
import {alsaUtils} from "./alsa-utils.ts";
import {dropbear} from "./dropbear.ts";
import {bash} from "./bash.ts";
import {e2fsprogs} from "./e2fsprogs.ts";
import {bison} from "./bison.ts";
import {cairo} from "./cairo.ts";
import {expat} from "./expat.ts";
import {pixman} from "./pixman.ts";
import {libpng} from "./libpng.ts";
import {freetype2} from "./freetype2.ts";
import {libjpeg} from "./libjpeg.ts";
import {pcre} from "./pcre.ts";
import {libxml2} from "./libxml2.ts";
import {wpa_supplicant} from "./wpa_supplicant.ts";
import {libinput} from "./libinput.ts";
import {libsndfile} from "./sndfile.ts";
import {mtdev} from "./mtdev.ts";
import {libevdev} from "./libevdev.ts";
import {pulseaudio} from "./pulseaudio.ts";
import {iproute2} from "./iproute2.ts";
import {kmod, procps, utils} from "./util-linux.ts";
import {libmnl} from "./libmnl.ts";
import {glib} from "./glib.ts";
import {gettext} from "./gettext.ts";
import {libxcrypt} from "./libxcrypt.ts";
import {libffi} from "./libffi.ts";
import {xkbcommon} from "./optional/xkbcommon.ts";
import {pam} from "./pam.ts";
import {lua} from "./lua.ts";
import {ltdl} from "./ltdl.ts";
import {mgba} from "./mgba.ts";
import {adb} from "./adb.ts";
import {rg35xxpFirmware} from "./rg35xxp-firmware.ts";
import {skeleton} from "./skeleton.ts";
import {mpv} from "./mpv.ts";
import {libass} from "./libass.ts";
import {fribidi} from "./fribidi.ts";
import {harfbuzz} from "./harfbuzz.ts";
import {all} from "./all.ts";
import {uboot} from "./u-boot.ts";
import {moonlightEmbedded} from "./moonlight-embedded.ts";
import {opus} from "./opus.ts";
import {curl} from "./curl.ts";
import {avahi} from "./avahi.ts";
import {libevent} from "./libevent.ts";
import {libdaemon} from "./libdaemon.ts";
import {libxcb} from "./optional/xcb-xkb.ts";
import {xau} from "./optional/xau.ts";
import {xproto} from "./optional/xproto.ts";
import {tinyalsa} from "./optional/tinyalsa.ts";
import {SDLImage} from "./sdl2-image.ts";
import {SDLGfx} from "./sdl2-gfx.ts";
import {SDLTTF} from "./sdl2-ttf.ts";
import {cjson} from "./cjson.ts";
import {SDLMixer} from "./sdl2-mixer.ts";
import {qt5} from "./qt5.ts";
import {nss} from "./nss.ts";
import {icu} from "./icu.ts";
import {boost} from "./boost.ts";
import {librime} from "./librime.ts";
import {yaml} from "./yaml.ts";
import {opencc} from "./opencc.ts";
import {marisa} from "./marisa.ts";
import {leveldb} from "./leveldb.ts";
import {glog} from "./glog.ts";
import {yamlCPP} from "./yaml-cpp.ts";
import rg35xxp from "./rg35xxp-apps.ts";
import {ppsspp} from "./ppsspp.ts";
import {love2d} from "./love2d.ts";
import {luajit} from "./luajit.ts";
import {openal} from "./openal.ts";
import {libmodplug, libmpg123} from "./libmodplug.ts";
import {ogg, theora, vorbis} from "./xiph.ts";
import uutils from "./uutils.ts";
import {iw} from "./iw.ts";
import {tmux} from "./tmux.ts";
import {rgtv} from "./rgtv.ts";
import {miyoopod} from "./miyoopod.ts";

register(all)
register(avahi)
register(RetroArch)
register(RetroArchAssets)
register(cjson)
register(SDL2)
register(SDL2ALL);
register(SDLImage);
register(SDLMixer);
register(SDLTTF);
register(SDLGfx);
register(t507_gpu_drivers)
register(gamepad)
register(eudev)
register(busybox)
register(evtest)
register(openssh)
register(rsync)
register(openssl)
register(zlib)
register(bzip2)
register(libnl)
register(readline)
register(connman)
register(libnftnl)
register(gnutls)
register(nettle)
register(gmp)
register(iptables)
register(ncurses)
register(ffmpeg)
register(x264)
register(x265)
register(fontconfig)
register(notoCJK)
register(sqlite3)
register(libdrm)
register(dbus)
register(alsa)
register(alsaUtils)
register(tinyalsa)
register(dropbear)
register(bash)
register(e2fsprogs)
register(bison)
register(cairo)
register(expat)
register(pixman)
register(libpng)
register(libjpeg)
register(pcre)
register(libxml2)
register(wpa_supplicant)
register(utils)
register(kmod)
register(procps)
register(freetype2)
register(libinput)
register(libsndfile)
register(mtdev)
register(libevdev)
register(pulseaudio)
register(iproute2)
register(libmnl)
register(glib)
register(gettext)
register(libxcrypt)
register(xau)
register(xproto)
register(libxcb)
register(xkbcommon)
register(libffi)
register(pam)
register(lua)
register(ltdl)
register(mGCore)
register(mgba)
register(adb)
register(rg35xxpFirmware)
register(skeleton)
register(mpv)
register(libass)
register(fribidi)
register(harfbuzz)
register(uboot)
register(moonlightEmbedded)
register(opus)
register(curl)
register(libevent)
register(libdaemon)
register(qt5)
register(nss)
register(icu)
register(boost)
register(librime)
register(yaml)
register(yamlCPP)
register(opencc)
register(marisa)
register(glog)
register(leveldb)
register(rg35xxp.init)
register(rg35xxp.launcher)
register(rg35xxp.guard)
register(uutils.coreutils)
register(uutils.findutils)
register(ppsspp)
register(love2d)
register(luajit)
register(openal)
register(libmodplug)
register(ogg)
register(vorbis)
register(theora)
register(libmpg123)
register(iw)
register(tmux)
register(rgtv)
register(miyoopod)
