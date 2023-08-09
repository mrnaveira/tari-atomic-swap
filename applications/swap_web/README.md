## Getting started

### tari-connector npm dependency
The `tari-connector` package must be installed in the global `npm` folder. To do it:
* Clone the `tari-connector` repository
* Run `npm install`
* Run `npm link`


### Run the Tari Atomic Swap web
With all previous prerequisites in order, in this project's folder (`swap-web`) run:
```
npm install
npm link tari-connector
npm run dev
```