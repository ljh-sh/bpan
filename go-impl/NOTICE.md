# NOTICE

bpan
Copyright 2026 Li Junhao (ljh-sh)

This product includes software developed at
The Baidu Netdisk Open Platform team (https://github.com/baidu-netdisk).

---

## Vendored component

The following component is included in this distribution via `git subtree`:

- **baidu-netdisk/baidu-drive-sdk-go** (Apache-2.0)
  - Upstream: https://github.com/baidu-netdisk/baidu-drive-sdk-go
  - Vendored location: `./upstream/baidu-drive-sdk-go/`
  - Vendored commit (squashed): see `./upstream/BAIDU-DRIVE-SDK-README.md`
  - License: Apache-2.0 (full text: `./upstream/baidu-drive-sdk-go/LICENSE`)

We chose `git subtree` so the build is reproducible from this repository
alone — no network fetch is needed to compile, and the community can
patch and release at its own cadence.

---

## License

This project is licensed under the Apache License, Version 2.0.
You may obtain a copy of the License at:

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.