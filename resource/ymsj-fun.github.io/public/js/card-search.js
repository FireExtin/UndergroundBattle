var allCards = {};
var activeCards = [];
var shown_count = 0;
var keywords = {};

var filter = {
  "str": _ => true,
  "cost": _ => true,
  "color": _ => true,
  "set": _ => true,
  "magic": _ => true,
  "typeb": _ => true,
}

$(document).ready(() => {
  $.getJSON("/cards/cards.json", (json) => {
    // this will show the info it in firebug console
    allCards = json;
    activeCards = Object.values(allCards);

    $('#ucq-count').text(`共查询到 ${activeCards.length} 张牌。`)
    sortActiveCards();
    showMoreCards();
  });
  $.getJSON("/public/docs/keywords.json", (json) => {
    keywords = json;
  });
});

$(window).scroll(() => {
  if ($(window).scrollTop() + $(window).height() >= $('#underworld-cards-query').height()) {
    showMoreCards();
  }
});

function sortActiveCards() {
  activeCards = activeCards.sort((a, b) => {
    if (a['deckcard'] != b['deckcard'])
      return b['deckcard'] - a['deckcard'];
    if (a['set'] != b['set']) {
      const setmap = {
        "基础": 0, "序曲": 1, "洛城": 2,
        "间奏": 3, "帷幕": 4, "星光": 5,
        "传奇": 6, "霸权": 7
      };
      return setmap[b['set']] - setmap[a['set']];
    }
    return a['set-id'] - b['set-id'];
  }
  );
}

function showMoreCards() {
  let ucq_cards = $('#ucq-cards');
  for (let i = 0; i < 30; i++) {
    if (shown_count >= activeCards.length) {
      $('#ucq-show').text('木有嘞！');
      return;
    }

    let card = activeCards[shown_count];
    ucq_cards.append(
      $('<li></li>').addClass('card')
        .css('background-image', `url('/cards/compressed/${card.id} ${card.name}.jpg'`)
        .attr('data-index', `${shown_count}`)
        .click(() => showCardInfo(`${card.id}`))
    )
    shown_count++;
  }
}

// cost filter
function costSelect(id) {
  let n = parseInt(id);

  for (var i = -1; i <= 7; i++) {
    if (i == n) $(`#filter-cost-button-${i}`).addClass('active');
    else $(`#filter-cost-button-${i}`).removeClass('active');
  }

  if (n == -1)
    filter.cost = _ => true
  else if (n == 7)
    filter.cost = card => card.cost && parseInt(card.cost) >= 7
  else
    filter.cost = card => card.cost && parseInt(card.cost) == n
}

// color filter
function colorSelect(id) {
  let n = {
    "任意色": 0, "黄": 1, "绿": 2, "蓝": 3, "红": 4,
    "灰": 5, "白": 6, "黑": 7, "紫": 8, "中立": 9,
  }[id];

  for (var i = 0; i <= 9; i++) {
    if (i == n) $(`#filter-color-button-${i}`).addClass('active');
    else $(`#filter-color-button-${i}`).removeClass('active');
  }

  if (n == 0) filter.color = (_ => true);
  else filter.color = (card => card.color && card.color == id);
}

// set filter
function setSelect(id) {
  let [n, name] = {
    "Any": [0, ""], "JC": [1, "基础"], "XQ": [2, "序曲"],
    "LC": [3, "洛城"], "JZ": [4, "间奏"],
    "WM": [5, "帷幕"], "XG": [6, "星光"],
    "CQ": [7, "传奇"], "BQ": [8, "霸权"],
  }[id];

  for (var i = 0; i <= 8; i++) {
    if (i == n) $(`#filter-set-button-${i}`).addClass('active');
    else $(`#filter-set-button-${i}`).removeClass('active');
  }

  if (n == 0) filter.set = (_ => true);
  else filter.set = (card => card.set == name);
}

function magicSelect(id) {
  let n = {
    "任意魔法": 0, "心灵": 1, "神圣": 2, "星辰": 3,
    "死亡": 4, "鲜血": 5, "无魔法": 6,
  }[id];

  for (var i = 0; i <= 6; i++) {
    if (i == n) $(`#filter-magic-button-${i}`).addClass('active');
    else $(`#filter-magic-button-${i}`).removeClass('active');
  }

  if (n == 0)
    filter.magic = _ => true;
  else if (n == 6)
    filter.magic = card => !card.magic || card.magic == "";
  else
    filter.magic = card => card.magic && card.magic == id;
}

function typeSelect(id) {
  let n = {
    "任意": 0, "角色": 1, "事务": 2, "附属": 3, "秘社": 4, "地区": 5, "任务": 6
  }[id];

  for (var i = 0; i <= 6; i++) {
    if (i == n) $(`#filter-type-button-${i}`).addClass('active');
    else $(`#filter-type-button-${i}`).removeClass('active');
  }

  if (n == 0)
    filter.typeb = _ => true
  else
    filter.typeb = card => card["basic-type"].includes(id)
}

function updateTextFilter() {
  var t = document.getElementById("uw-search").value;
  var words = t.split(" ");
  filter.str = (card) => {
    var b = true;
    for (var i = 0; i < words.length; i++) {
      b &= card["search"].includes(words[i]);
    }
    return b;
  };
}

function filterCards() {
  updateTextFilter();

  $('#ucq-cards').empty();
  activeCards = [];
  shown_count = 0;

  for (let k in allCards) {
    let card = allCards[k];

    if (card.istoken && !$('#opt-token').prop('checked'))
      continue;

    if (filter.str(card) &&
      filter.typeb(card) &&
      filter.cost(card) &&
      filter.color(card) &&
      filter.set(card) &&
      filter.magic(card)) {
      activeCards.push(card);
    }
  }

  $('#ucq-count').text(`共查询到 ${activeCards.length} 张牌。`)
  $('#ucq-show').text('点击查看更多');

  sortActiveCards();
  showMoreCards();
}

// Hide the card information block
$(document).mouseup(function (e) {
  if (e.button != 0) return;
  var cpib = $('#cp-info');
  if (!cpib.is(e.target) && cpib.has(e.target).length === 0) {
    $('#card-preview').fadeOut(300);
  }
});

// Show the card information block
function showCardInfo(id) {
  let card = allCards[id];

  // set card image
  $('#cp-card').css('background-image', `url("/cards/${id} ${card.name}.jpg")`)

  // set card info
  let info = $('#cp-info');
  info.empty();

  // info: name; set|id 
  info.append($(`<div class="name">${card.name}</div>`))
    .append($(`<div class="setname"> ${card.set} | ${card.id} </div>`));

  var item;

  // info: [color] type
  item = $('<div class="type"></div>')
  if (card.color) item.text(` ${card.color}${card.type} `);
  else item.text(` ${card.type} `);
  info.append(item);

  // info: [cost][lyl~]
  item = $('<div class="attr"></div>');
  if (card.cost) item.html(`费用：${card.cost} `);
  // if (card.lyl) card.lyl.forEach(lyl => item.append(`${lyl} `));
  if (card.lyl) item.append(`${card.lyl}`);
  info.append(item);

  // info: [dfc] [abl] [magic]
  item = $('<div class="attr"></div>');
  if (card.dfc)
    item.html(`防御：${card.dfc}&nbsp;&nbsp;`);
  if (card.abl && card.abl != "")
    item.append(`能力：${card.abl}&nbsp;&nbsp;`);
  if (card.magic && card.magic != "")
    item.append(`魔法：${card.magic}`);
  info.append(item);

  // info: [req] [sc]
  item = $('<div class="attr"></div>');
  if (card.req) item.html(`势力值：${card.req}&nbsp;&nbsp;`);
  if (card.sc) item.append(`分数：${card.sc}`);
  info.append(item);

  // info: [hands]
  item = $('<div class="attr"></div>');
  if (card.hands) item.html(`起始手牌数量：${card.hands}`);
  info.append(item);

  // info: text
  var text = card.text;
  if ($('#opt-kws').prop('checked') && card.keywords.length > 0) {
    text += "\n\n";
    for (let kw of card.keywords) {
      console.log(kw);
      text += '> ' + keywords[kw] + "\n";
    }
  }

  info.append($(`<textarea class="text" readonly>${text}</textarea>`));

  $('#card-preview').fadeIn(300); //.css('display', 'block');
}
