import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";

export const boost: Pack = {
    name: "boost",
    version: "1.87.0",
    description: "Boost provides free peer-reviewed portable C++ source libraries.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: 'https://cfhcable.dl.sourceforge.net/project/boost/boost/1.87.0/boost_1_87_0.tar.gz',
            filename: `boost_1_87_0.tar.gz`
        });
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        const boost = build('boost_1_87_0')
        await $`[ ! -e ${boost} ] || rm -r ${boost}`;
        await $`tar -xf ${tar(`boost_1_87_0.tar.gz`)}`.cwd(build());

        await $`./bootstrap.sh --prefix=${rootfs('usr')} --with-libraries=all`.cwd(boost);
        await $`sed -i 's/using gcc ;/using gcc : aarch64 : aarch64-linux-gnu-g++ ;/' project-config.jam`.cwd(boost);
        const boost_args = [
            'toolset=gcc-aarch64',
            'link=shared',
            'variant=release',
            'threading=multi'
        ]
        await $`./b2 -j16 ${boost_args} install`.cwd(boost);
    }
}
