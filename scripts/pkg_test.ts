import {type Pack, packages, register, resolve} from './pkg';
import {beforeEach, describe, expect, test} from "bun:test";

describe('resolve function', () => {
    beforeEach(() => {
        // Clear the packages array before each test
        while (packages.length > 0) {
            packages.pop();
        }
    });

    test('should resolve dependencies correctly', () => {
        const packA: Pack = {
            name: 'A',
            version: '1.0.0',
            description: 'Package A',
            enable: true,
            dependencies: [],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        const packB: Pack = {
            name: 'B',
            version: '1.0.0',
            description: 'Package B',
            enable: true,
            dependencies: [{name: 'A', version: '1.0.0', required: true}],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        register(packB);
        register(packA);

        const result = resolve();
        expect(result).toEqual([packA, packB]);
    });

    test('should throw error for missing required dependency', () => {
        const packA: Pack = {
            name: 'A',
            version: '1.0.0',
            description: 'Package A',
            enable: true,
            dependencies: [{name: 'B', version: '1.0.0', required: true}],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        register(packA);

        expect(() => resolve()).toThrow('Required dependency B not found for package A');
    });

    test('should ignore optional dependencies if not found', () => {
        const packA: Pack = {
            name: 'A',
            version: '1.0.0',
            description: 'Package A',
            enable: true,
            dependencies: [{name: 'B', version: '1.0.0', required: false}],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        register(packA);

        const result = resolve();
        expect(result).toEqual([packA]);
    });

    test('should not register duplicate packages', () => {
        const packA: Pack = {
            name: 'A',
            version: '1.0.0',
            description: 'Package A',
            enable: true,
            dependencies: [],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        register(packA);
        expect(() => register(packA)).toThrow('package A already exists');
    });

    test('should resolve specific package and its dependencies', () => {
        const packA: Pack = {
            name: 'A',
            version: '1.0.0',
            description: 'Package A',
            enable: true,
            dependencies: [],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        const packB: Pack = {
            name: 'B',
            version: '1.0.0',
            description: 'Package B',
            enable: true,
            dependencies: [{name: 'A', version: '1.0.0', required: true}],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        register(packA);
        register(packB);

        const result = resolve('B');
        expect(result).toEqual([packA, packB]);
    });

    test('should throw error if specific package not found', () => {
        expect(() => resolve('NonExistentPackage')).toThrow('Package NonExistentPackage not found');
    });

    test('should not resolve packages with enable set to false', () => {
        const packA: Pack = {
            name: 'A',
            version: '1.0.0',
            description: 'Package A',
            enable: false,
            dependencies: [],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        register(packA);

        const result = resolve();
        expect(result).toEqual([]);
    });

    test('should resolve package B and its dependency A', () => {
        const packA: Pack = {
            name: 'A',
            version: '1.0.0',
            description: 'Package A',
            enable: true,
            dependencies: [],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        const packB: Pack = {
            name: 'B',
            version: '1.0.0',
            description: 'Package B',
            enable: true,
            dependencies: [{name: 'A', version: '1.0.0', required: true}],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        const packC: Pack = {
            name: 'C',
            version: '1.0.0',
            description: 'Package C',
            enable: true,
            dependencies: [],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        register(packA);
        register(packB);
        register(packC);

        const result = resolve('B');
        expect(result).toEqual([packA, packB]);
    });
    test('should resolve dependencies correctly with version check', () => {
        const packA: Pack = {
            name: 'A',
            version: '1.0.0',
            description: 'Package A',
            enable: true,
            dependencies: [],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        const packB: Pack = {
            name: 'B',
            version: '1.0.0',
            description: 'Package B',
            enable: true,
            dependencies: [{name: 'A', version: '^1.0.0', required: true}],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        register(packA);
        register(packB);

        const result = resolve();
        expect(result).toEqual([packA, packB]);
    });

    test('should throw error for missing required dependency with version check', () => {
        const packA: Pack = {
            name: 'A',
            version: '1.0.0',
            description: 'Package A',
            enable: true,
            dependencies: [{name: 'B', version: '^1.0.0', required: true}],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        register(packA);

        expect(() => resolve()).toThrow('Required dependency B not found for package A');
    });

    test('should ignore optional dependencies if version does not match', () => {
        const packA: Pack = {
            name: 'A',
            version: '1.0.0',
            description: 'Package A',
            enable: true,
            dependencies: [{name: 'B', version: '^2.0.0', required: false}],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        register(packA);

        const result = resolve();
        expect(result).toEqual([packA]);
    });

    test('should resolve specific package and its dependencies with version check', () => {
        const packA: Pack = {
            name: 'A',
            version: '1.0.0',
            description: 'Package A',
            enable: true,
            dependencies: [],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        const packB: Pack = {
            name: 'B',
            version: '1.0.0',
            description: 'Package B',
            enable: true,
            dependencies: [{name: 'A', version: '^1.0.0', required: true}],
            sync: async () => {
            },
            make: async () => {
            },
            install: async () => {
            },
            clean: async () => {
            }
        };

        register(packA);
        register(packB);

        const result = resolve('B');
        expect(result).toEqual([packA, packB]);
    });
});
