import fs from 'node:fs';import path from 'node:path';import{pathToFileURL}from'node:url';
const source=process.cwd(),root=path.dirname(process.argv[1]);const{writeCurrentFixture}=await import(pathToFileURL(path.join(source,'tools/current-native-source.mjs')));
for(const[name,file]of[['bq030-web','web/src/game/bq030Native112.fixture.json'],['bq030-sites','sites/test/fixtures/bq030-native-v0112.json'],['jc052-worker','sites/test/fixtures/jc052-native-v0112.json']]){
 writeCurrentFixture('rust-game-wasm/tests/current112/'+name+'.json.gz',JSON.parse(fs.readFileSync(path.join(root,'raw-selectors',file),'utf8')));console.log('fresh Native packet',name);
}
