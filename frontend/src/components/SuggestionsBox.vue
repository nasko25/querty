<template>
    <div id = "suggestions" :class = "{hidden: isSuggestHidden}">
        <ul id = "suggestions-list">
            <li class = "suggestion" v-for = "suggestion in suggestions" :key = "suggestion">
                {{ suggestion }}
            </li>
        </ul>
    </div>
</template>

<script>

export default {
    name: 'SuggestionsBox',
    props: {
        suggestions: Array,
        isSuggestHidden: Boolean
    }
};

</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>

#suggestions {
    height: 210px;
    width: 80%;
    max-width: 654px;
    min-width: 380px;

 --suggestions-background: #124;

    background-color: var(--suggestions-background);
    position: absolute;
    margin-left: auto;
    margin-right: auto;
    left: 0;
    right: 0;
    top: 435px;
    z-index: 0;
    border-radius: 0 0 20px 20px;
}

#suggestions-list {
    background-color: var(--suggestions-background);
    margin-top: 1.5em;
    padding-left: 20px;
    border-radius: 0 0 20px 20px;
}

.suggestion {
    background-color: var(--suggestions-background);
    list-style: none;
    text-align: left;
    padding-top: 0.4em;
    border-radius: 0 0 20px 20px;
}

.suggestion::before {
    content: " ";
    display: inline-block;
    height: 13px;
    width: 13px;
    background-image: url("~@/assets/search-green.png");
    background-size:contain;
    background-repeat:no-repeat;
    position: relative;
    top: 3px;
    padding-right: 10px;
}
</style>
