/*
 * Copyright 2022 Google Inc. All Rights Reserved.
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *     http://www.apache.org/licenses/LICENSE-2.0
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// Note: our JS should have been generated in
// `[out-dir]/snippets/wasm-bindgen-rayon-[hash]/workerHelpers.worker.js`,
// resolve the main module via `../../..`.
//
// This might need updating if the generated structure changes on wasm-bindgen
// side ever in the future, but works well with bundlers today. The whole
// point of this crate, after all, is to abstract away unstable features
// and temporary bugs so that you don't need to deal with them in your code.
import initWbg, { wbg_rayon_start_worker } from '../../../';

onmessage = async ({ data: { receiver, ...initData } }) => {
  await initWbg(initData);
  postMessage(true);
  wbg_rayon_start_worker(receiver);
};
