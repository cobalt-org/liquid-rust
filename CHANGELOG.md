<a name="0.18.2"></a>
## 0.18.2 (2019-02-01)


#### Features

* **jekyll-filter:**  slugify filter ([21a5be0b](https://github.com/cobalt-org/liquid-rust/commit/21a5be0b0f538ae67d31e5a23180f88af95df69d))



<a name="0.18.1"></a>
## 0.18.1 (2019-01-23)


#### Bug Fixes

* **comment:**  parse tags inside comment, but ignore their content ([a153b127](https://github.com/cobalt-org/liquid-rust/commit/a153b12775bc0d8c23f23905da60ea2c8f21dbee))
* **grammar:**  allow unmatched `}}` and `%}` as valid liquid ([1889c7b0](https://github.com/cobalt-org/liquid-rust/commit/1889c7b09e19f315e470ff2e70a06e503759eaa0), closes [#320](https://github.com/cobalt-org/liquid-rust/issues/320))
* **parser:**  blocks can accept invalid liquid ([3b2b5fcc](https://github.com/cobalt-org/liquid-rust/commit/3b2b5fcccd0bdec041ce09da9619cd837a81af88), closes [#277](https://github.com/cobalt-org/liquid-rust/issues/277))



<a name="0.18.0"></a>
## 0.18.0 (2018-12-27)


#### Behavior Features

*   Indexing by variable ([c216a439](https://github.com/cobalt-org/liquid-rust/commit/c216a439d978bedb88ec4baba0e8703d6877e20e), closes [#209](https://github.com/cobalt-org/liquid-rust/issues/209))
* **array:**  indexing with `.first` and `.last` ([36d79cf2](https://github.com/cobalt-org/liquid-rust/commit/36d79cf2f5855b2f5428d8a2d173af8d26dd98bf))
* **case_block:**  support comma separated values in `when` ([0e56f772](https://github.com/cobalt-org/liquid-rust/commit/0e56f7721863a8065ee33eb035b343e16d04e231))
* **errors:**  Report available tags blocks ([04e486a8](https://github.com/cobalt-org/liquid-rust/commit/04e486a87781c6cb46c8ca0da56f7a310eb567cb), closes [#183](https://github.com/cobalt-org/liquid-rust/issues/183))
* **filters:**
  *  array manipulation filters ([94e66600](https://github.com/cobalt-org/liquid-rust/commit/94e6660040d5b021e0ee6aac86ef51e25dd2c725))
  *  Allow the input to index ([ceccb9b2](https://github.com/cobalt-org/liquid-rust/commit/ceccb9b28f68ea168a75daa13059e776ca0d880c), closes [#207](https://github.com/cobalt-org/liquid-rust/issues/207))
  *  add filters "at_least" and "at_most" ([be3e55c0](https://github.com/cobalt-org/liquid-rust/commit/be3e55c079fe43f8f35ebe5add00fd05ef912f79))
* **for-block:**
  * Support iterating on Object ([2469bfc0](https://github.com/cobalt-org/liquid-rust/commit/2469bfc0678b75e16e466cc73b7761b1eb27658d), closes [#201](https://github.com/cobalt-org/liquid-rust/issues/201))
  *  support parameters with variables ([7376ccf5](https://github.com/cobalt-org/liquid-rust/commit/7376ccf51f5d2f0a67ca0dbad3d25d395f4fbd6d), closes [#162](https://github.com/cobalt-org/liquid-rust/issues/162))
* **if_block:**  support multiple conditions with `and` and `or` ([fb16a066](https://github.com/cobalt-org/liquid-rust/commit/fb16a066eeb7fd226883fdae64ec34660d8e539d))
* **tablerow:**  add tablerow object for its tag ([6b95cca5](https://github.com/cobalt-org/liquid-rust/commit/6b95cca5486e2b351bda5b8ffd9e23df98a478d1))
* **unless_block:**  support `else` ([8577eb1e](https://github.com/cobalt-org/liquid-rust/commit/8577eb1e63091623e80da732e3bbf4f6106329d1))
* **tags:**  add more shopify tags ([18660736](https://github.com/cobalt-org/liquid-rust/commit/1866073682eb97a62bae8eefef87dcc44740a0e6), closes [#163](https://github.com/cobalt-org/liquid-rust/issues/163))
* **grammar:**
  *  Support for empty/blank literals ([ef721815](https://github.com/cobalt-org/liquid-rust/commit/ef72181575e46c7c9329deb30ecb651cab903d3f), closes [#222](https://github.com/cobalt-org/liquid-rust/issues/222)
  *  Support for nil literals ([7d3b0e5b](https://github.com/cobalt-org/liquid-rust/commit/7d3b0e5bb82bef9cc82698fc9514a47a557fdb6f), closes [#223](https://github.com/cobalt-org/liquid-rust/issues/223)
*   Improve error reporting ([e373b1e1](https://github.com/cobalt-org/liquid-rust/commit/e373b1e1a7a1597fe29b5c9020beae1612fc1002))

#### Bug Fixes

*   Deeply nested array indexes ([51c3a853](https://github.com/cobalt-org/liquid-rust/commit/51c3a853a74d0b933983e95a2c6f38d1fdf6512d), closes [#230](https://github.com/cobalt-org/liquid-rust/issues/230))
*   Support more expressive indexing ([e579dd3d](https://github.com/cobalt-org/liquid-rust/commit/e579dd3df4465a4d3b42e3d06293f370ef5b750c)
* **for_block:**  make ranges inclusive. ([42055c35](https://github.com/cobalt-org/liquid-rust/commit/42055c356125b0e66524a5ee7a2fa34e4af4397c))
* **if:**  Improve accuracy of contains op ([07452cd3](https://github.com/cobalt-org/liquid-rust/commit/07452cd38f8d8907bd1f5f602439e2ee99021e69))
* **newlines_to_br:**  should preserve newlines ([01904edb](https://github.com/cobalt-org/liquid-rust/commit/01904edbbb2bc8d7279fe82274df939f7e75d857))
* **comment_block:**  allow nesting ([8d1f64e7](https://github.com/cobalt-org/liquid-rust/commit/8d1f64e7a4e749cf402fe77d2321cba4dacc8a6b))
* **errors:**
  * List alternative filters ([406187bc](https://github.com/cobalt-org/liquid-rust/commit/406187bcd1963636d7119d751938a37b6ce5bab5)
  * look of values/variables ([111160c3](https://github.com/cobalt-org/liquid-rust/commit/111160c3898195753cbccbdca693870acdc278dc), closes [#258](https://github.com/cobalt-org/liquid-rust/issues/258))
* **dbg:**  Fix --help for debug tool ([45c5d397](https://github.com/cobalt-org/liquid-rust/commit/45c5d39770081fc88b1570c64b03d56e6d53b45d))

#### Breaking Behavior Changes

* **grammar:**
  *  Support for empty/blank literals ([ef721815](https://github.com/cobalt-org/liquid-rust/commit/ef72181575e46c7c9329deb30ecb651cab903d3f), closes [#222](https://github.com/cobalt-org/liquid-rust/issues/222)
  *  Support for nil literals ([7d3b0e5b](https://github.com/cobalt-org/liquid-rust/commit/7d3b0e5bb82bef9cc82698fc9514a47a557fdb6f), closes [#223](https://github.com/cobalt-org/liquid-rust/issues/223)

#### Performance

*   Slight change for if-existence ([92aaadf5](https://github.com/cobalt-org/liquid-rust/commit/92aaadf5a583adaad08745613842d9b98225a14a))
*   if-existence bypass error reporting cost ([c7fde6f4](https://github.com/cobalt-org/liquid-rust/commit/c7fde6f44c96a23a86b8fb0f29d060e4dec389bc))
*   Improve for-loop ([f4500fdf](https://github.com/cobalt-org/liquid-rust/commit/f4500fdfafa8543b9161fbefba3175acfcf0b23c))
*   Slight speed up for for-over-hash ([8e2ce0e6](https://github.com/cobalt-org/liquid-rust/commit/8e2ce0e66bfebc698589f5ca9c87853f4cc80170))
*   Speed up variable accesses ([f7392486](https://github.com/cobalt-org/liquid-rust/commit/f7392486b540646a05080275b4d4d5cfa507e3c5)
*   Reduce allocations ([cbb1d254](https://github.com/cobalt-org/liquid-rust/commit/cbb1d254b0ed15b32674e7688af6c55cee91c125), closes [#188](https://github.com/cobalt-org/liquid-rust/issues/188)
* **render:**
  * Use a Write ([0093a595](https://github.com/cobalt-org/liquid-rust/commit/0093a595b9dc0f335c3f8eed7a0309123a82b708), closes [#187](https://github.com/cobalt-org/liquid-rust/issues/187)
  * Bypass UTF-8 validation overhead ([c759fc33](https://github.com/cobalt-org/liquid-rust/commit/c759fc335710797c0cfd75c42b0c598d191fc0b6))
  * Default buffer size ([58eec66b](https://github.com/cobalt-org/liquid-rust/commit/58eec66b49a2a7156b745753c50080b8afad6b7a))
* **value:**
  *  Support str's ([e3aae68d](https://github.com/cobalt-org/liquid-rust/commit/e3aae68d672570db89aeea3daf58423b8f9e6bda)
  *  Reduce allocations with `Cow` ([7fd1e62d](https://github.com/cobalt-org/liquid-rust/commit/7fd1e62d7622ea5e61ef52f225b897f318d59b2c)
  * Allow slicing Paths ([9601e30a](https://github.com/cobalt-org/liquid-rust/commit/9601e30a0803261e92a84bfe86699108f13c03d7)

#### API Features

* **parser:**  accept newlines as `WHITESPACE` ([7bec9871](https://github.com/cobalt-org/liquid-rust/commit/7bec9871b47e15e1aa07a40d3de3518bd80303fc), closes [#286](https://github.com/cobalt-org/liquid-rust/issues/286), [#280](https://github.com/cobalt-org/liquid-rust/issues/280))
* **interpreter:**
  * Runtime partials ([0ef46a17](https://github.com/cobalt-org/liquid-rust/commit/0ef46a170aa21c6113830fa62c26e8b708b97fff))
  *  Support runtime include for tags ([5a0854fa](https://github.com/cobalt-org/liquid-rust/commit/5a0854fa3706505fd1b32dc1bf6ecb7292e173a4)
  *  Allow named stack frames ([4c378178](https://github.com/cobalt-org/liquid-rust/commit/4c3781782176c3ea334963c50f52f9119b178017))
  *  Create dedicated Path for indexing into a Value ([a936ba52](https://github.com/cobalt-org/liquid-rust/commit/a936ba5290130d1fe53ccb4b59c2d04744406674))
  * New caching policies ([d2ba7a61](https://github.com/cobalt-org/liquid-rust/commit/d2ba7a61c0af0600bfd8841fc6f12951ae53557c))
  *  Support arbitrary state ([033c9b75](https://github.com/cobalt-org/liquid-rust/commit/033c9b75c7476057f0858e78ec5b93a0a7cb7895)
* **error:**
  *  liquid re-export all error stuff ([808f708e](https://github.com/cobalt-org/liquid-rust/commit/808f708e5ac48853a977a3cdfc98ec968116766e))
  *  Cloneable errors ([e18c68e1](https://github.com/cobalt-org/liquid-rust/commit/e18c68e14cc3a894737ef23e0f07a9363eea5f79))
  *  Improve missing variable errors ([d6e1aea5](https://github.com/cobalt-org/liquid-rust/commit/d6e1aea5ff9f0340c87bc45b4c07fcd34991805c))
* **value:**
  *  Convinience eq/cmp impls ([78f7a952](https://github.com/cobalt-org/liquid-rust/commit/78f7a9522fc9693919777b04bc8400e708870701))
  *  Value literal macro ([ea5ac0aa](https://github.com/cobalt-org/liquid-rust/commit/ea5ac0aaaa976da089fac6b025bfea450e4da852))
  *  Allow moving into constituent types ([bc07812c](https://github.com/cobalt-org/liquid-rust/commit/bc07812ccdb06e0383cc4701831d0fc20d5be654))
  *  Create to_value ([61ae6de6](https://github.com/cobalt-org/liquid-rust/commit/61ae6de625c53171327263bafec954f0ee2a4977))
  * Return error from Globals ([fde1397b](https://github.com/cobalt-org/liquid-rust/commit/fde1397b5d81b015e30379432c8c017acb63c3b3))
  * Rich Gobals API ([385a62fd](https://github.com/cobalt-org/liquid-rust/commit/385a62fd6d362c4cb5a58694dcf073a471920c34)

#### Breaking API Changes

*   Speed up variable accesses ([f7392486](https://github.com/cobalt-org/liquid-rust/commit/f7392486b540646a05080275b4d4d5cfa507e3c5)
*   Allow slicing Paths ([9601e30a](https://github.com/cobalt-org/liquid-rust/commit/9601e30a0803261e92a84bfe86699108f13c03d7)
*   Rich Gobals API ([385a62fd](https://github.com/cobalt-org/liquid-rust/commit/385a62fd6d362c4cb5a58694dcf073a471920c34)
*   Support more expressive indexing ([e579dd3d](https://github.com/cobalt-org/liquid-rust/commit/e579dd3df4465a4d3b42e3d06293f370ef5b750c)
*   Force serde to always be on ([7f1e2027](https://github.com/cobalt-org/liquid-rust/commit/7f1e2027c28cfd8e34e9ac7e98efd50867c5052d)
*   Cleanup each crate's API ([2e4ab661](https://github.com/cobalt-org/liquid-rust/commit/2e4ab66116ee97183b4d712006bba459f02b3b88)
*   Isolate context creation ([00cac8cb](https://github.com/cobalt-org/liquid-rust/commit/00cac8cb1804e3e1c0514673a9ecc23a13a5f006)
*   Isolate Stack state ([3f8e9432](https://github.com/cobalt-org/liquid-rust/commit/3f8e9432e481fb67ee82d1120405f00112f60810)
*   Isolate cycle state ([34dc950a](https://github.com/cobalt-org/liquid-rust/commit/34dc950a3eddd54340910029d6b4bf13e27721b0)
*   Isolate interrupt state ([06596643](https://github.com/cobalt-org/liquid-rust/commit/06596643a46ec40eadba000602f4af7aa81fe2a4)
*   Reduce allocations ([cbb1d254](https://github.com/cobalt-org/liquid-rust/commit/cbb1d254b0ed15b32674e7688af6c55cee91c125), closes [#188](https://github.com/cobalt-org/liquid-rust/issues/188)
* **compiler:**
  *  Rename LiquidOptions ([0442f38c](https://github.com/cobalt-org/liquid-rust/commit/0442f38c83ec12cd93f6e906c4c43d38383d08bb)
  *  Move filters to interpreter ([b9c2ff87](https://github.com/cobalt-org/liquid-rust/commit/b9c2ff87caf5c30ac6fe5520a9e343690d020c7a)
* **context:**  Reduce scope of public API ([866eb0cb](https://github.com/cobalt-org/liquid-rust/commit/866eb0cb3758a919919009a421c92cd8a148906d)
* **error:**
  *  Simplify Result::context API ([6e6f3b5e](https://github.com/cobalt-org/liquid-rust/commit/6e6f3b5eec626a3e9fd325754f36e622f760021e)
  *  Simplify by removing cloning ([bbd71146](https://github.com/cobalt-org/liquid-rust/commit/bbd71146b4b8f7a14f6ac131b981d76f3dae5a5e)
  *  Clean up error API ([6a950048](https://github.com/cobalt-org/liquid-rust/commit/6a9500488bf6c3bbcc0cbc8c09f52f694595cb6f)
* **errors:**  List alternative filters ([406187bc](https://github.com/cobalt-org/liquid-rust/commit/406187bcd1963636d7119d751938a37b6ce5bab5)
* **filter:**  Switch to standard error type ([3d18b718](https://github.com/cobalt-org/liquid-rust/commit/3d18b71865b38d9f0a3aa4722d0a7adad13c2016)
* **interpreter:**
  *  Support runtime include for tags ([5a0854fa](https://github.com/cobalt-org/liquid-rust/commit/5a0854fa3706505fd1b32dc1bf6ecb7292e173a4)
  *  Moving Text closer to use ([6e9f5bec](https://github.com/cobalt-org/liquid-rust/commit/6e9f5bec78ec8295352b4d40230234bdc89966ee)
  *  Rename Globals to ValueStore ([fbe4f2c3](https://github.com/cobalt-org/liquid-rust/commit/fbe4f2c3f4953ccb08ee317d2e4be4755d5d0e62)
  *  Clarify names ([6b96f92b](https://github.com/cobalt-org/liquid-rust/commit/6b96f92be169aad4d9393cb48d9ad37e856d014a)
* **perf:**  Don't clone globals ([fbc1c153](https://github.com/cobalt-org/liquid-rust/commit/fbc1c153df9988db09cbb8a0a148c8a30b9ab598)
* **plugins:**
  *  Support arbitrary state ([033c9b75](https://github.com/cobalt-org/liquid-rust/commit/033c9b75c7476057f0858e78ec5b93a0a7cb7895)
  *  Abstract plugin registry ([ef4cabf3](https://github.com/cobalt-org/liquid-rust/commit/ef4cabf3c7cdf885bccd1f61d9dfa3da9aab5fa2)
* **render:**  Use a Write ([0093a595](https://github.com/cobalt-org/liquid-rust/commit/0093a595b9dc0f335c3f8eed7a0309123a82b708), closes [#187](https://github.com/cobalt-org/liquid-rust/issues/187)
* **value:**
  *  Newtype for Map ([eab6f40f](https://github.com/cobalt-org/liquid-rust/commit/eab6f40fdb1a6f5858b0dbe3cf89d89600559fd1)
  *  Support str's ([e3aae68d](https://github.com/cobalt-org/liquid-rust/commit/e3aae68d672570db89aeea3daf58423b8f9e6bda)
  *  Reduce allocations with `Cow` ([7fd1e62d](https://github.com/cobalt-org/liquid-rust/commit/7fd1e62d7622ea5e61ef52f225b897f318d59b2c)



<a name="0.17.1"></a>
## 0.17.1 (2018-11-17)


#### Bug Fixes

*   Deeply nested array indexes ([51c3a853](https://github.com/cobalt-org/liquid-rust/commit/51c3a853a74d0b933983e95a2c6f38d1fdf6512d), closes [#230](https://github.com/cobalt-org/liquid-rust/issues/230))
* **error:**  Improve error reporting ([e372470d](https://github.com/cobalt-org/liquid-rust/commit/e372470d2ed030a4294b7781ce8e80b8041ce673))

#### Features

* **array:**
  * indexing with `.first` and `.last` ([36d79cf2](https://github.com/cobalt-org/liquid-rust/commit/36d79cf2f5855b2f5428d8a2d173af8d26dd98bf))
  * array manipulation filters ([94e66600](https://github.com/cobalt-org/liquid-rust/commit/94e6660040d5b021e0ee6aac86ef51e25dd2c725))
* **blocks:**
  * support multiple if conditions with `and` and `or` ([fb16a066](https://github.com/cobalt-org/liquid-rust/commit/fb16a066eeb7fd226883fdae64ec34660d8e539d))
  * add tablerow object for its tag ([6b95cca5](https://github.com/cobalt-org/liquid-rust/commit/6b95cca5486e2b351bda5b8ffd9e23df98a478d1))



<a name="0.17.0"></a>
## 0.17.0 (2018-10-18)


#### Breaking Changes

*   Support more expressive indexing ([e579dd3d](https://github.com/cobalt-org/liquid-rust/commit/e579dd3df4465a4d3b42e3d06293f370ef5b750c)
* **for_block:**  make ranges inclusive. ([42055c35](https://github.com/cobalt-org/liquid-rust/commit/42055c356125b0e66524a5ee7a2fa34e4af4397c))

#### Features

* Indexing by variable ([c216a439](https://github.com/cobalt-org/liquid-rust/commit/c216a439d978bedb88ec4baba0e8703d6877e20e), closes [#209](https://github.com/cobalt-org/liquid-rust/issues/209))
* **for_block:**  support parameters with variables ([7376ccf5](https://github.com/cobalt-org/liquid-rust/commit/7376ccf51f5d2f0a67ca0dbad3d25d395f4fbd6d), closes [#162](https://github.com/cobalt-org/liquid-rust/issues/162))
* **filters:**
  * Added: "at_most" ([be3e55c0](https://github.com/cobalt-org/liquid-rust/commit/be3e55c079fe43f8f35ebe5add00fd05ef912f79))
  * Added: "at_least" ([be3e55c0](https://github.com/cobalt-org/liquid-rust/commit/be3e55c079fe43f8f35ebe5add00fd05ef912f79))
* **tags:**
  * Added tablerow ([18660736](https://github.com/cobalt-org/liquid-rust/commit/1866073682eb97a62bae8eefef87dcc44740a0e6), closes [#163](https://github.com/cobalt-org/liquid-rust/issues/163))
  * Added ifchanged ([18660736](https://github.com/cobalt-org/liquid-rust/commit/1866073682eb97a62bae8eefef87dcc44740a0e6), closes [#163](https://github.com/cobalt-org/liquid-rust/issues/163))
  * Added increment ([18660736](https://github.com/cobalt-org/liquid-rust/commit/1866073682eb97a62bae8eefef87dcc44740a0e6), closes [#163](https://github.com/cobalt-org/liquid-rust/issues/163))
  * Added decrement ([18660736](https://github.com/cobalt-org/liquid-rust/commit/1866073682eb97a62bae8eefef87dcc44740a0e6), closes [#163](https://github.com/cobalt-org/liquid-rust/issues/163))

#### Bug Fixes

* **for_block:**  make ranges inclusive. ([42055c35](https://github.com/cobalt-org/liquid-rust/commit/42055c356125b0e66524a5ee7a2fa34e4af4397c))



<a name="0.16.1"></a>
## 0.16.1 (2018-10-05)




<a name="0.16.0"></a>
## 0.16.0 (2018-10-04)


#### Breaking Changes

* Force serde to always be on ([7f1e2027](https://github.com/cobalt-org/liquid-rust/commit/7f1e2027c28cfd8e34e9ac7e98efd50867c5052d))
* **context:**  Reduce scope of public API ([866eb0cb](https://github.com/cobalt-org/liquid-rust/commit/866eb0cb3758a919919009a421c92cd8a148906d))
  * Isolate context creation ([00cac8cb](https://github.com/cobalt-org/liquid-rust/commit/00cac8cb1804e3e1c0514673a9ecc23a13a5f006))
  * Isolate Stack state ([3f8e9432](https://github.com/cobalt-org/liquid-rust/commit/3f8e9432e481fb67ee82d1120405f00112f60810))
  * Isolate cycle state ([34dc950a](https://github.com/cobalt-org/liquid-rust/commit/34dc950a3eddd54340910029d6b4bf13e27721b0))
  * Isolate interrupt state ([06596643](https://github.com/cobalt-org/liquid-rust/commit/06596643a46ec40eadba000602f4af7aa81fe2a4))
* **error:**  Clean up error API ([6a950048](https://github.com/cobalt-org/liquid-rust/commit/6a9500488bf6c3bbcc0cbc8c09f52f694595cb6f))
  * **filter:**  Switch to standard error type ([3d18b718](https://github.com/cobalt-org/liquid-rust/commit/3d18b71865b38d9f0a3aa4722d0a7adad13c2016))
* Cleanup each crate's API ([2e4ab661](https://github.com/cobalt-org/liquid-rust/commit/2e4ab66116ee97183b4d712006bba459f02b3b88))
  * Reduce allocations ([cbb1d254](https://github.com/cobalt-org/liquid-rust/commit/cbb1d254b0ed15b32674e7688af6c55cee91c125), closes [#188](https://github.com/cobalt-org/liquid-rust/issues/188))
* **interpreter:**  Clarify names ([6b96f92b](https://github.com/cobalt-org/liquid-rust/commit/6b96f92be169aad4d9393cb48d9ad37e856d014a))
* **perf:**  Don't clone globals ([fbc1c153](https://github.com/cobalt-org/liquid-rust/commit/fbc1c153df9988db09cbb8a0a148c8a30b9ab598))
* **render:**  Use a Write ([0093a595](https://github.com/cobalt-org/liquid-rust/commit/0093a595b9dc0f335c3f8eed7a0309123a82b708), closes [#187](https://github.com/cobalt-org/liquid-rust/issues/187))
* **value:**
  *  Support str's ([e3aae68d](https://github.com/cobalt-org/liquid-rust/commit/e3aae68d672570db89aeea3daf58423b8f9e6bda))
  *  Reduce allocations with `Cow` ([7fd1e62d](https://github.com/cobalt-org/liquid-rust/commit/7fd1e62d7622ea5e61ef52f225b897f318d59b2c))

#### Features

* **for-block:**  Support iterating on Object ([2469bfc0](https://github.com/cobalt-org/liquid-rust/commit/2469bfc0678b75e16e466cc73b7761b1eb27658d), closes [#201](https://github.com/cobalt-org/liquid-rust/issues/201))
* **value:**  Create `to_value` ([61ae6de6](https://github.com/cobalt-org/liquid-rust/commit/61ae6de625c53171327263bafec954f0ee2a4977))

#### Performance

* Reduce allocations ([cbb1d254](https://github.com/cobalt-org/liquid-rust/commit/cbb1d254b0ed15b32674e7688af6c55cee91c125), closes [#188](https://github.com/cobalt-org/liquid-rust/issues/188))
* **render:**  Use a Write ([0093a595](https://github.com/cobalt-org/liquid-rust/commit/0093a595b9dc0f335c3f8eed7a0309123a82b708), closes [#187](https://github.com/cobalt-org/liquid-rust/issues/187))
* **value:**  Support `&'static str`'s ([e3aae68d](https://github.com/cobalt-org/liquid-rust/commit/e3aae68d672570db89aeea3daf58423b8f9e6bda)) ([7fd1e62d](https://github.com/cobalt-org/liquid-rust/commit/7fd1e62d7622ea5e61ef52f225b897f318d59b2c))


<a name="0.15.0"></a>
## 0.15.0 (2018-07-30)


#### Breaking Changes

*   Upgrade from f32 to f64 ([3eddded2](https://github.com/cobalt-org/liquid-rust/commit/3eddded24056c9f5c2d2d2f3adf143809fe82507))

#### Features

*   Expose filters/tags ([027a67cc](https://github.com/cobalt-org/liquid-rust/commit/027a67cccb9b40ffac0e25d5d9cd4501bdbe4d63))
*   Upgrade from f32 to f64 ([3eddded2](https://github.com/cobalt-org/liquid-rust/commit/3eddded24056c9f5c2d2d2f3adf143809fe82507))
* **date:**  Support today/now ([6a1e0a0f](https://github.com/cobalt-org/liquid-rust/commit/6a1e0a0f3ddc7892e8c84597929dbebc4dd80d29), closes [#181](https://github.com/cobalt-org/liquid-rust/issues/181))



<a name="0.14.3"></a>
## 0.14.3 (2018-04-10)


#### Bug Fixes

*   Reduce deps for users ([41e9b01a](https://github.com/cobalt-org/liquid-rust/commit/41e9b01a6b2925562b2ef073a8a420c64f08e570))
* **error:**  Make API consumable by failure ([54be3400](https://github.com/cobalt-org/liquid-rust/commit/54be3400dcebe4944196a36be9c99a5187a6f550))



<a name="0.14.2"></a>
## 0.14.2 (2018-03-16)


#### Features

* **if:**  Bare if is an existence check ([7ab091ca](https://github.com/cobalt-org/liquid-rust/commit/7ab091cadce48d4cb066b3c494fd26f34f0d9625))



<a name="0.14.1"></a>
## 0.14.1 (2018-01-24)


#### Features

* **API:**
  *  Support &String->Scalar ([b87c983c](https://github.com/cobalt-org/liquid-rust/commit/b87c983c1b5fc9061c1d86424b135119d82fe737))
  *  Re-export datetime ([1ca16f5a](https://github.com/cobalt-org/liquid-rust/commit/1ca16f5a90769e427f45e743fdbfd47629e1d178))



<a name="0.14.0"></a>
## 0.14.0 (2018-01-22)


#### Features

API
* **Value:**  Control key order ([7ff0fcd0](https://github.com/cobalt-org/liquid-rust/commit/7ff0fcd04d2570aea5338e03128de62e494bee62), closes [#159](https://github.com/cobalt-org/liquid-rust/issues/159))

Users
* **errors:**
  *  Provide context on compile errors ([c17df1f3](https://github.com/cobalt-org/liquid-rust/commit/c17df1f30b2eec8c0ded04919c73c9d5a1d63377))
  *  Report context during render ([73e26cf7](https://github.com/cobalt-org/liquid-rust/commit/73e26cf786a5883a5fe98d66678134392f107cda), closes [#105](https://github.com/cobalt-org/liquid-rust/issues/105))
* **filters:**  Implement basic `compact` support ([c0eadd5c](https://github.com/cobalt-org/liquid-rust/commit/c0eadd5c384e6d5745036ed344116d137043c154))

#### Breaking Changes

API
*   Reduce string cloning ([3d93928b](https://github.com/cobalt-org/liquid-rust/commit/3d93928b2a9ac378dbb3ca8bd097b1ed7112620f)

Users
* **value:**  Improve value coercion practices ([ebb4f40e](https://github.com/cobalt-org/liquid-rust/commit/ebb4f40e315c280825e74cad60b4cd91bbe06ea0), closes [#99](https://github.com/cobalt-org/liquid-rust/issues/99)

#### Performance

*   Reduce string cloning ([3d93928b](https://github.com/cobalt-org/liquid-rust/commit/3d93928b2a9ac378dbb3ca8bd097b1ed7112620f)

#### Bug Fixes

API
*   Remove warning when no-default-features ([8c43de87](https://github.com/cobalt-org/liquid-rust/commit/8c43de871b437d14ac8da14d283bc906c6dea9f2))

Users
* **filters:**  date_in_tz can't parse cobalt date ([1dae5276](https://github.com/cobalt-org/liquid-rust/commit/1dae52767680c7a2b628f631078a97d1ef37ca37))
* **value:**  Improve value coercion practices ([ebb4f40e](https://github.com/cobalt-org/liquid-rust/commit/ebb4f40e315c280825e74cad60b4cd91bbe06ea0), closes [#99](https://github.com/cobalt-org/liquid-rust/issues/99)



<a name="0.13.7"></a>
## 0.13.7 (2018-01-10)


#### Features

*   Implement `contains` operator ([a0d27205](https://github.com/cobalt-org/liquid-rust/commit/a0d2720570d13d489d7d929452c41334a9d019eb), closes [#155](https://github.com/cobalt-org/liquid-rust/issues/155))



<a name="0.13.6"></a>
## 0.13.6 (2017-12-29)


#### Features

* **filters:**  date can parse YYYY-MM-DD HH:MM:SS TTTT ([59ab76dc](https://github.com/cobalt-org/liquid-rust/commit/59ab76dcd343a6d9d0fff497e6ba2ff1140b3f2a))



<a name="0.13.5"></a>
## 0.13.5 (2017-12-27)

* Update dependencies

<a name="0.13.4"></a>
## 0.13.4 (2017-12-27)


#### Bug Fixes

* **parse:**  Error on empty expressions ([5cffe44a](https://github.com/cobalt-org/liquid-rust/commit/5cffe44a5fb3821dab8a41b8662596421f387659), closes [#139](https://github.com/cobalt-org/liquid-rust/issues/139))
* **raw:**  Stop swapping the text's order ([bd45c14b](https://github.com/cobalt-org/liquid-rust/commit/bd45c14b58e1b22e156b42f3c5629e3a0692e7d4), closes [#79](https://github.com/cobalt-org/liquid-rust/issues/79))



<a name="0.13.3"></a>
## 0.13.3 (2017-12-18)


#### Bug Fixes

* **for:**  Re-enable support for object.access ([cc9998b5](https://github.com/cobalt-org/liquid-rust/commit/cc9998b55a225941fc5d402f414c32abf64c4500))



<a name="0.13.2"></a>
## 0.13.2 (2017-12-18)


#### Features

* **api:**  Add missing traits ([e0f82705](https://github.com/cobalt-org/liquid-rust/commit/e0f82705e25e7ff40d246749e7d8b0da04637813))

#### Bug Fixes

* **nil:**  Equality logic missed a case ([111d10a6](https://github.com/cobalt-org/liquid-rust/commit/111d10a695eaf8d906c77569aac627042d52f8eb))



<a name="0.13.1"></a>
## 0.13.1 (2017-12-17)

Minor docs change.


<a name="0.13.0"></a>
## 0.13.0 (2017-12-17)


#### Features

* **api:**  Make Renderable debuggable ([802b0af0](https://github.com/cobalt-org/liquid-rust/commit/802b0af0045874565d68a4c4f3b957ddef1b44bd))

#### Bug Fixes

* **dbg:**  Remove debug code ([7bf2a3d4](https://github.com/cobalt-org/liquid-rust/commit/7bf2a3d4754252a0c67c7c514e1dca542e565e4c))
* **for:**  Remove non-standard for_loop variable ([0d9515fe](https://github.com/cobalt-org/liquid-rust/commit/0d9515fe1a8c89e9604beb1a69370256d0f23f08))

#### Breaking Changes

* **for:**  Remove non-standard for_loop variable ([0d9515fe](https://github.com/cobalt-org/liquid-rust/commit/0d9515fe1a8c89e9604beb1a69370256d0f23f08))



<a name="0.12.0"></a>
## 0.12.0 (2017-11-29)


#### Features

*   Make LiquidOptions cloneable ([838e5261](https://github.com/cobalt-org/liquid-rust/commit/838e5261b6654aab2a93cb5ff2220f75e2d554df))
  *   Make TemplateRepository cloneable ([94f337ae](https://github.com/cobalt-org/liquid-rust/commit/94f337aee53cdd126001b32427b415b20d70d25a))
  *   Make ParseBlock cloneable ([472fb638](https://github.com/cobalt-org/liquid-rust/commit/472fb638e79ab1126979aecb258990d4b93f2935))
  *   Make ParseTag cloneable ([ec59839d](https://github.com/cobalt-org/liquid-rust/commit/ec59839d9d1deff52bb663d0310d5efbca5acace))

#### Breaking Change

*   Make TemplateRepository cloneable ([94f337ae](https://github.com/cobalt-org/liquid-rust/commit/94f337aee53cdd126001b32427b415b20d70d25a))
*   Make ParseBlock cloneable ([472fb638](https://github.com/cobalt-org/liquid-rust/commit/472fb638e79ab1126979aecb258990d4b93f2935))
*   Make ParseTag cloneable ([ec59839d](https://github.com/cobalt-org/liquid-rust/commit/ec59839d9d1deff52bb663d0310d5efbca5acace))


<a name="0.11.0"></a>
## 0.11.0 (2017-11-08)


#### Features

* **syntax:** Add `arr[0]` and `obj["name"]` indexing (PR #141, fixes #127)
* **value:**  Add nil value to support foreign data (PR #140, [89f6660d](https://github.com/cobalt-org/liquid-rust/commit/89f6660d61ee3a59d3e29e7ad8fe6b31791b1d6f))

#### Breaking Change

* **value:**  Add nil value to support foreign data (PR #140, [89f6660d](https://github.com/cobalt-org/liquid-rust/commit/89f6660d61ee3a59d3e29e7ad8fe6b31791b1d6f))
  * Technically will break anyone matching on `liquid::Value`.

<a name="0.10.1"></a>
## 0.10.1 (2017-09-24)


#### Features

*   Turn `serde` into a default feature. ([6be99f1d](https://github.com/cobalt-org/liquid-rust/commit/6be99f1da4c066dc08eafd6918f604409f93d43d), closes [#128](https://github.com/cobalt-org/liquid-rust/issues/128))

### Bug Fixes
* Stop recompiling everytime due to Skeptic.


<a name="v0.10.0"></a>
## v0.10.0 (2017-05-27)


#### Features

* **filters:**
  *  Add sort_natural ([ef14f871](https://github.com/cobalt-org/liquid-rust/commit/ef14f87151d73e6079450ec46ebd9da805966aa7))
  *  Implement a dummy `compact` ([44d4d061](https://github.com/cobalt-org/liquid-rust/commit/44d4d0619754fbce519a8d51743651d4cee8e00d))
  *  map filter ([52dc03c0](https://github.com/cobalt-org/liquid-rust/commit/52dc03c06a25a037cc65da3f39f46711be62d76c))
  *  Add concat filter ([36d0d2c1](https://github.com/cobalt-org/liquid-rust/commit/36d0d2c1c4250fa16a3a16af2754ba14f6adb62d))
  *  `round` accepts a precision param ([ef691f13](https://github.com/cobalt-org/liquid-rust/commit/ef691f137d6327df7479abd68ae165f282da2aff))
* **Value:**
  *  Add serde support ([8ae7f5a1](https://github.com/cobalt-org/liquid-rust/commit/8ae7f5a1da00434a6c4d7297938164452d943f09) and [839f44b3](https://github.com/cobalt-org/liquid-rust/commit/839f44b3bdce926c8520d77e9a9e35b60d8e522a), closes [#113](https://github.com/cobalt-org/liquid-rust/issues/113))
  *  Add convenience functions ([4b73b3c2](https://github.com/cobalt-org/liquid-rust/commit/4b73b3c2ebb2a48c05052adff8a104187d58943f))
  *  Publicly expose Object and Array ([280c6d99](https://github.com/cobalt-org/liquid-rust/commit/280c6d9956347f7903e719cb55ee14da46ce1465))
* **debug:**  Adding CLI for testing liquid ([171cbfe0](https://github.com/cobalt-org/liquid-rust/commit/171cbfe0ba297c496dbb738ba136b8d6cbce9eb7) and [9d4b4088](https://github.com/cobalt-org/liquid-rust/commit/9d4b408881292cb57c858d144b91a3f626e53f05))
* **performance:**  Add benchmarks ([0e90972d](https://github.com/cobalt-org/liquid-rust/commit/0e90972d620c02f6e587076e093c330287de070b))

#### Bug Fixes

* **filters:**
  *  Align behavior with shopify/liquid ([ebd7ebc6](https://github.com/cobalt-org/liquid-rust/commit/ebd7ebc696b6176e6a8f24b3efb58f5683d1c341))
  *  Moved `pluralize` to `extra-filters` ([17d57c09](https://github.com/cobalt-org/liquid-rust/commit/17d57c093fc8771531c13b6f587b44b2b25d2b03))



