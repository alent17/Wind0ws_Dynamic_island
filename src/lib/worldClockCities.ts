import { CLOCK_TIME_ZONES } from "$lib/timeZones";
import type { ResolvedLocale } from "$lib/i18n";

export type WorldClockCity = { id:string; timeZone:string; names:Record<ResolvedLocale,string> };

// Keep cities sharing a time zone distinct so each can be found by name.
export const WORLD_CLOCK_CITIES: WorldClockCity[] = [
  ...CLOCK_TIME_ZONES.map(zone => ({id:zone.value,timeZone:zone.value,names:zone.names})),
  {id:"istanbul",timeZone:"Europe/Istanbul",names:{"zh-CN":"伊斯坦布尔",en:"Istanbul",ja:"イスタンブール"}},
  {id:"shanghai",timeZone:"Asia/Shanghai",names:{"zh-CN":"上海",en:"Shanghai",ja:"上海"}},
  {id:"guangzhou",timeZone:"Asia/Shanghai",names:{"zh-CN":"广州",en:"Guangzhou",ja:"広州"}},
  {id:"shenzhen",timeZone:"Asia/Shanghai",names:{"zh-CN":"深圳",en:"Shenzhen",ja:"深圳"}},
  {id:"yangon",timeZone:"Asia/Yangon",names:{"zh-CN":"仰光",en:"Yangon",ja:"ヤンゴン"}},
  {id:"riyadh",timeZone:"Asia/Riyadh",names:{"zh-CN":"利雅得",en:"Riyadh",ja:"リヤド"}},
  {id:"lima",timeZone:"America/Lima",names:{"zh-CN":"利马",en:"Lima",ja:"リマ"}},
  {id:"karachi",timeZone:"Asia/Karachi",names:{"zh-CN":"卡拉奇",en:"Karachi",ja:"カラチ"}},
  {id:"kuala-lumpur",timeZone:"Asia/Kuala_Lumpur",names:{"zh-CN":"吉隆坡",en:"Kuala Lumpur",ja:"クアラルンプール"}},
  {id:"santiago",timeZone:"America/Santiago",names:{"zh-CN":"圣地亚哥",en:"Santiago",ja:"サンティアゴ"}},
  {id:"mexico-city",timeZone:"America/Mexico_City",names:{"zh-CN":"墨西哥城",en:"Mexico City",ja:"メキシコシティ"}},
  {id:"osaka",timeZone:"Asia/Tokyo",names:{"zh-CN":"大阪",en:"Osaka",ja:"大阪"}},
  {id:"lisbon",timeZone:"Europe/Lisbon",names:{"zh-CN":"里斯本",en:"Lisbon",ja:"リスボン"}},
  {id:"delhi",timeZone:"Asia/Kolkata",names:{"zh-CN":"德里",en:"Delhi",ja:"デリー"}},
  {id:"jakarta",timeZone:"Asia/Jakarta",names:{"zh-CN":"雅加达",en:"Jakarta",ja:"ジャカルタ"}},
  {id:"taipei",timeZone:"Asia/Taipei",names:{"zh-CN":"中国台北",en:"Taipei, China",ja:"中国・台北"}},
  {id:"manila",timeZone:"Asia/Manila",names:{"zh-CN":"马尼拉",en:"Manila",ja:"マニラ"}},
  {id:"bogota",timeZone:"America/Bogota",names:{"zh-CN":"波哥大",en:"Bogotá",ja:"ボゴタ"}},
  {id:"rome",timeZone:"Europe/Rome",names:{"zh-CN":"罗马",en:"Rome",ja:"ローマ"}},
  {id:"prague",timeZone:"Europe/Prague",names:{"zh-CN":"布拉格",en:"Prague",ja:"プラハ"}},
  {id:"brussels",timeZone:"Europe/Brussels",names:{"zh-CN":"布鲁塞尔",en:"Brussels",ja:"ブリュッセル"}},
  {id:"vienna",timeZone:"Europe/Vienna",names:{"zh-CN":"维也纳",en:"Vienna",ja:"ウィーン"}},
  {id:"helsinki",timeZone:"Europe/Helsinki",names:{"zh-CN":"赫尔辛基",en:"Helsinki",ja:"ヘルシンキ"}},
  {id:"lagos",timeZone:"Africa/Lagos",names:{"zh-CN":"拉各斯",en:"Lagos",ja:"ラゴス"}},
  {id:"nairobi",timeZone:"Africa/Nairobi",names:{"zh-CN":"内罗毕",en:"Nairobi",ja:"ナイロビ"}},
  {id:"dhaka",timeZone:"Asia/Dhaka",names:{"zh-CN":"达卡",en:"Dhaka",ja:"ダッカ"}},
  {id:"amsterdam",timeZone:"Europe/Amsterdam",names:{"zh-CN":"阿姆斯特丹",en:"Amsterdam",ja:"アムステルダム"}},
  {id:"dublin",timeZone:"Europe/Dublin",names:{"zh-CN":"都柏林",en:"Dublin",ja:"ダブリン"}},
  {id:"buenos-aires",timeZone:"America/Argentina/Buenos_Aires",names:{"zh-CN":"布宜诺斯艾利斯",en:"Buenos Aires",ja:"ブエノスアイレス"}},
  {id:"casablanca",timeZone:"Africa/Casablanca",names:{"zh-CN":"卡萨布兰卡",en:"Casablanca",ja:"カサブランカ"}},
  {id:"johannesburg",timeZone:"Africa/Johannesburg",names:{"zh-CN":"约翰内斯堡",en:"Johannesburg",ja:"ヨハネスブルグ"}},
];

export function resolveWorldClockZone(timeZone:string): string {
  return timeZone === "system" ? Intl.DateTimeFormat().resolvedOptions().timeZone : timeZone;
}
