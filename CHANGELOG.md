# Changelog

## [0.7.1](https://github.com/dabay6/yak-analyzer/compare/v0.7.0...v0.7.1) (2026-11-15)


### Bug Fixes

* **entries:** read a tool's configuration as the tool reads it, and check the named repositories ([9866e16](https://github.com/dabay6/yak-analyzer/commit/9866e167a9123eff40cb1c00a3ff3e9285c268e6))
* **entries:** read what a tool's configuration names as the tool reads it ([12d3a14](https://github.com/dabay6/yak-analyzer/commit/12d3a14fd832b3b7e45014deeb10ba2eab7e8fb5))

## [0.7.0](https://github.com/dabay6/yak-analyzer/compare/v0.6.0...v0.7.0) (2026-11-15)


### Features

* **audit:** compare with a base commit and mark findings new or inherited ([e3788ac](https://github.com/dabay6/yak-analyzer/commit/e3788ac0f14be1b423dd337d73ab0d92e0274090))
* **audit:** compare with a base commit and mark findings new or inherited ([37696f5](https://github.com/dabay6/yak-analyzer/commit/37696f5e8f75ef955e6e84a4b00e0d76e74e1161))
* **audit:** end with the launcher and keep deep projects off a small stack ([3046e13](https://github.com/dabay6/yak-analyzer/commit/3046e136545c054b89b93ab62b084b6b57e4439d))
* **audit:** follow renames and compare cycles by their edges ([5705c76](https://github.com/dabay6/yak-analyzer/commit/5705c7645d584f0f0c86c5ce67dc7ac75e581915))

## [0.6.0](https://github.com/dabay6/yak-analyzer/compare/v0.5.0...v0.6.0) (2026-11-15)


### Features

* **audit:** add the audit command and its SARIF report ([0a2244c](https://github.com/dabay6/yak-analyzer/commit/0a2244cd330a12033e742234b0dd76c4e4458c7a))
* **audit:** add the audit command and report the five dead-code kinds ([a0b4dce](https://github.com/dabay6/yak-analyzer/commit/a0b4dcec4e9b86954fc341e2eee666e71991568c))
* **audit:** report unresolved imports and circular dependencies ([101044e](https://github.com/dabay6/yak-analyzer/commit/101044e14912bcbf7826fc85a103cffdc31c7315))
* **audit:** report unused files, exports and types ([71fc1be](https://github.com/dabay6/yak-analyzer/commit/71fc1be6d76f6455527772dea0ed1e355a6b2224))

## [0.5.0](https://github.com/dabay6/yak-analyzer/compare/v0.4.0...v0.5.0) (2026-11-14)


### Features

* **uses:** read each file's names and count the uses of its exports ([ee5beed](https://github.com/dabay6/yak-analyzer/commit/ee5beed9c83b12ca0beedd04d70c26919fab5ca5))
* **uses:** read each file's names and count the uses of its exports ([05f8f37](https://github.com/dabay6/yak-analyzer/commit/05f8f37dc9fc47ca8df977d5aea6bd608ac1be38))

## [0.4.0](https://github.com/dabay6/yak-analyzer/compare/v0.3.0...v0.4.0) (2026-11-14)


### Features

* **entries:** read entry points from scripts and tool configuration ([194e0a3](https://github.com/dabay6/yak-analyzer/commit/194e0a3172849864eb908829b2485d938f4440f8))
* **entries:** read entry points from scripts and tool configuration ([8dc96ff](https://github.com/dabay6/yak-analyzer/commit/8dc96ff66de80a0c2bb4c86567ba78b3c2feea73))

## [0.3.0](https://github.com/dabay6/yak-analyzer/compare/v0.2.0...v0.3.0) (2026-11-14)


### Features

* **entries:** find entry points from standard files and declarations ([480603b](https://github.com/dabay6/yak-analyzer/commit/480603bd0411b099c12327dbac9b56ef676cd299))
* **entries:** find entry points from standard files and declarations ([6f8f15d](https://github.com/dabay6/yak-analyzer/commit/6f8f15d8ef0929f4e9d7ed1ca5e113c8ff43896a))

## [0.2.0](https://github.com/dabay6/yak-analyzer/compare/v0.1.0...v0.2.0) (2026-11-13)


### Features

* **cli:** add the interface and graph commands ([a2e27b9](https://github.com/dabay6/yak-analyzer/commit/a2e27b9367498a61f4d6636d7c58d6ce33000736))
* **files:** read the file list through git ([b22794e](https://github.com/dabay6/yak-analyzer/commit/b22794e29ed1accfebb7edb7c85a83ec4e87908b))
* **hand-off:** read the hand-off file and check its declarations ([a2a87eb](https://github.com/dabay6/yak-analyzer/commit/a2a87ebdf252d25d2854b99cce83b6221da5e73d))
* **imports:** parse scripts and find their imports ([a030afe](https://github.com/dabay6/yak-analyzer/commit/a030afe1133fbb32601a6bc87a8a29c1005f1086))
* **packages:** read the declarations' marks and the packages ([7c8dd39](https://github.com/dabay6/yak-analyzer/commit/7c8dd394c18ff25787066ade4dfc1e17eff831dc))
* **pages:** read Vue components, MDX documents and HTML pages ([82d6f4d](https://github.com/dabay6/yak-analyzer/commit/82d6f4dfa93711049cdb7c819552fada9b9b5bed))
* parse scripts and find their imports ([2e3f4bd](https://github.com/dabay6/yak-analyzer/commit/2e3f4bd402f14e8e14999c16d5c9b0e276302546))
* read Vue components, MDX documents and HTML pages ([ffa64fc](https://github.com/dabay6/yak-analyzer/commit/ffa64fc667489eff1aa3690015449e4b2a6265de))
* **resolve:** lead imports by a package's name and by # names to source ([2fdec08](https://github.com/dabay6/yak-analyzer/commit/2fdec0847561f40f730ad62a756d3a6eda1a1940))
* **resolve:** lead relative imports to files and name every place outside ([cf9c912](https://github.com/dabay6/yak-analyzer/commit/cf9c9128f10fd197de8b88136d4201cc6fb653d0))
* the file list, the declarations' marks and the packages ([3964f5b](https://github.com/dabay6/yak-analyzer/commit/3964f5bee96a96478d7b6ecdad34ee60702e952a))
* the interface and graph commands, and the hand-off file ([4b8ed3f](https://github.com/dabay6/yak-analyzer/commit/4b8ed3ff0ef1223518651cf0dcd225d8a0981111))
* **tsconfig:** read tsconfig files for paths, baseUrl and outDir ([48e6997](https://github.com/dabay6/yak-analyzer/commit/48e6997de6dd6ec877d16c759856d77c0cfdb245))
* where each import leads ([6a93af1](https://github.com/dabay6/yak-analyzer/commit/6a93af11fde41dc70ec31c6e38601c5a533b2fe3))

## 0.1.0 (2026-11-11)


### Features

* a walking skeleton that answers its version ([4b26927](https://github.com/dabay6/yak-analyzer/commit/4b26927e9056b1889bd80e008b291d22c3c8ccad))
* add a binary that answers its version ([6480b4b](https://github.com/dabay6/yak-analyzer/commit/6480b4b46a549bd82ac09436606653198a266e63))
* add the launcher and the six npm packages ([ff890d0](https://github.com/dabay6/yak-analyzer/commit/ff890d0d8236ae0d60586e0eff3684cad94f71a4))
