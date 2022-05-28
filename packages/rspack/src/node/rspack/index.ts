import createDebug from 'debug';
import type {
  RawOptions,
  ExternalObject,
  OnLoadContext,
  OnResolveContext,
  OnLoadResult,
  OnResolveResult,
} from '@rspack/binding';
import * as binding from '@rspack/binding';

import type { RspackPlugin } from './plugins';
import { validateRawOptions } from './options';

const debugRspack = createDebug('rspack');
const debugNapi = createDebug('napi');

binding.initCustomTraceSubscriber();

export type { RawOptions, OnLoadContext, OnResolveResult, OnLoadResult, OnResolveContext, RspackPlugin };

interface RspackOptions extends RawOptions {
  plugins?: RspackPlugin[];
}

interface RspackThreadsafeContext<T> {
  readonly id: number;
  readonly inner: T;
}

interface RspackThreadsafeResult<T> {
  readonly id: number;
  readonly inner: T;
}

const createDummyResult = (id: number): string => {
  const result: RspackThreadsafeResult<null> = {
    id,
    inner: null,
  };
  return JSON.stringify(result);
};

const isNil = (value: unknown): value is null | undefined => {
  return value === null || value === undefined;
};

class Rspack {
  #instance: ExternalObject<any>;
  lazyCompilerMap: Record<string, string>;
  constructor(public options: RspackOptions) {
    const { plugins = [], ...innerOptions } = options;
    validateRawOptions(innerOptions);
    debugRspack('rspack options', innerOptions);

    const isPluginExist = !!plugins.length;

    const buildStart = async (err: Error, value: string): Promise<string> => {
      if (err) {
        throw err;
      }

      const context: RspackThreadsafeContext<void> = JSON.parse(value);

      await Promise.all(plugins.map((plugin) => plugin.buildStart?.()));

      return createDummyResult(context.id);
    };

    const buildEnd = async (err: Error, value: string): Promise<string> => {
      if (err) {
        throw err;
      }

      const context: RspackThreadsafeContext<void> = JSON.parse(value);

      await Promise.all(plugins.map((plugin) => plugin.buildEnd?.()));

      return createDummyResult(context.id);
    };

    const load = async (err: Error, value: string): Promise<string> => {
      if (err) {
        throw err;
      }

      const context: RspackThreadsafeContext<OnLoadContext> = JSON.parse(value);

      for (const plugin of plugins) {
        const { id } = context.inner;
        const result = await plugin.load?.(id);
        debugNapi('onLoadResult', result, 'context', context);

        if (isNil(result)) {
          continue;
        }

        return JSON.stringify({
          id: context.id,
          inner: result,
        });
      }

      debugNapi('onLoadResult', null, 'context', context);

      return createDummyResult(context.id);
    };

    console.log('plugins:', plugins);

    const resolve = async (err: Error, value: string): Promise<string> => {
      if (err) {
        throw err;
      }

      const context: RspackThreadsafeContext<OnResolveContext> = JSON.parse(value);

      for (const plugin of plugins) {
        const { importer, importee } = context.inner;
        const result = await plugin.resolve?.(importee, importer);
        debugNapi('onResolveResult', result, 'context', context);

        if (isNil(result)) {
          continue;
        }

        return JSON.stringify({
          id: context.id,
          inner: result,
        });
      }

      debugNapi('onResolveResult', null, 'context', context);
      return createDummyResult(context.id);
    };
    console.log('raw options', options);

    this.#instance = binding.newRspack(
      JSON.stringify(options),
      isPluginExist
        ? {
            loadCallback: load,
            resolveCallback: resolve,
            buildStartCallback: buildStart,
            buildEndCallback: buildEnd,
          }
        : null
    );
  }

  async build() {
    const map = await binding.build(this.#instance);
    this.setLazyCompilerMap(map);
    return map;
  }

  async rebuild(changedFile: string[]) {
    const [diff, map] = await binding.rebuild(this.#instance, changedFile);
    this.setLazyCompilerMap(map);
    return diff;
  }

  setLazyCompilerMap(map) {
    for (const key in map) {
      const value = map[key];
      if (Object.values(this.options.entries).indexOf(value) > -1) {
        delete map[key];
      }
    }
    this.lazyCompilerMap = map;
  }

  lazyCompileredSet = new Set<string>();

  async lazyBuild(chunkName: string) {
    const filename = this.lazyCompilerMap[chunkName];
    if (filename && !this.lazyCompileredSet.has(filename)) {
      console.log('lazy compiler ', filename);
      this.lazyCompileredSet.add(filename);
      await this.rebuild([filename]);
    }
  }
}

export { Rspack };
export default Rspack;