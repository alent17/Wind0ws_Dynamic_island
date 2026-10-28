import {test, expect} from "@playwright/test";

test.use({viewport:{width:1600,height:900}});
test("weather extension saves up to three places and restores them", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const f=window as any;
    f.settingsOverrides={islandStyle:"edge",islandEdge:"top",language:"en",islandWidgets:{left:[],right:["weather"]},islandWidgetsEnabled:true,wheelVolumeControl:true};
    f.weatherCalls=[];
    const invoke=f.__TAURI_INTERNALS__.invoke;
    f.__TAURI_INTERNALS__.invoke=(command:string,args:any,options:any)=>{
      if(command === "search_weather_locations") return Promise.resolve([{name:args.query,country:"Country",admin1:"",admin2:"",latitude:({Alpha:10,Beta:20,Gamma:30} as any)[args.query] ?? 40,longitude:1}]);
      if(command === "get_weather_for_location") {f.weatherCalls.push(args.location);if(f.failWeather) return Promise.reject(new Error("offline"));return Promise.resolve({weatherTemperature:args.location.latitude,weatherCode:1,weatherUpdatedAt:1,weatherForecast:[{date:"2026-10-28",weatherCode:1,temperatureMax:25,temperatureMin:15}]});}
      return invoke(command,args,options);
    };
    f.mountApp();
  });
  await expect(page.locator(".fixed-host")).toBeVisible();
  await page.evaluate(() => (window as any).expand());
  const panel=page.locator('[data-widget-kind="weather"] .info-panel');
  await expect(panel).toBeVisible();
  await panel.hover(); await page.mouse.wheel(0,120);
  const ext=page.locator(".weather-extension");
  await expect(ext).toBeVisible();
  const input=ext.locator("input");
  for(const city of ["Alpha","Beta","Gamma"]) {
    await input.fill(city);await input.press("Enter");
    await ext.locator(".results button").first().click();
    await expect(ext.locator(".place").filter({hasText:city})).toBeVisible();
  }
  await expect(ext.locator(".place")).toHaveCount(3);
  await expect(ext.locator(".count")).toHaveText("3/3");
  await expect(ext.locator(".search button")).toBeDisabled();
  await expect(ext.locator(".place").first().locator(".conditions strong").first()).toHaveText("10°");
  await ext.locator("header button").first().click();
  await expect(ext).toHaveCount(0);
  await panel.hover();await page.mouse.wheel(0,120);
  await expect(ext.locator(".place")).toHaveCount(3);
  await expect(ext.locator(".place").first().locator(".conditions strong").first()).toHaveText("10°");
  await ext.locator(".place").first().locator(".remove").click();
  await expect(ext.locator(".place")).toHaveCount(2);
  await expect(ext.locator(".count")).toHaveText("2/3");
  await page.evaluate(() => (window as any).destroy());
  await page.evaluate(() => {(window as any).failWeather=true;(window as any).mountApp();});
  await expect(page.locator(".fixed-host")).toBeVisible();
  await page.evaluate(() => (window as any).expand());
  await expect(panel).toBeVisible();
  await panel.hover(); await page.mouse.wheel(0,120);
  await expect(ext.locator(".place")).toHaveCount(2);
  await expect(ext.locator(".place").first().locator(".state.error")).toBeVisible();
  await page.evaluate(() => (window as any).failWeather=false);
  await ext.locator(".place").first().locator(".refresh").click();
  await expect(ext.locator(".place").first().locator(".conditions strong").first()).toHaveText("20°");
  await page.evaluate(() => (window as any).destroy());
});
