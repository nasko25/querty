<template>
<div>
    <a href="/"> <img alt="Q" id="Q" src="../assets/q.svg"/> </a>
    <SearchBar/>
    <ul id = "results-list">
        <li class = "result" v-for="(result, index) in results" :key = "index">
            <!-- &hellip; is only added if the text was truncated -->
            <a class = "result-title" :href="result.url" @mouseover="underline" @mouseleave="deunderline"> {{ result.title === undefined || result.title === "" ? truncate(result.text, 25) + (result.text.length > 25 ? "&hellip;" : "") : result.title }} </a>
            <br/>
            <a class = "result-url" :href="result.url" @mouseover="underlineTitle" @mouseleave="removeUnderline"> {{ result.url }} </a>
            <br/>
            <div class = "result-text"> <p v-html="getText(result)"></p> </div>
            <br/> <br/>
        </li>
    </ul>
    <!-- If there is an error, it will be displayed in this div -->
    <div id = "error" :class = "{ hidden : err === null }"> <p> {{ err }} </p> </div>
</div>
</template>

<script>
import SearchBar from './SearchBar.vue';

export default  {
    name: 'Results',
    components: {
        SearchBar
    },
    data() {
        return {
            results: [],
            err: null
        }
    },
    beforeMount() {
        this.fetchResults();
    },
    methods: {
        fetchResults: function() {
            const query = this.$route.query.q;
            if (query === undefined || query === "") {
                console.error("Empty query.")
                // indicate error
                this.err = "Empty query";
                this.results = [];
            }
            else {
                this.err = null;
            }
            console.log("query:", query);

            // TODO also sort by rank
            fetch(`http://${window.location.hostname}:8000/query/${encodeURIComponent(query)}`, {
                    method: 'GET'
                })
                    .then(response => {
                        if (response.ok)
                            response.json()
                                .then(response => this.results = response)
                                .catch(err => console.error(err));
                        else
                            console.error("Suggest served responsed with an unexpected code: " + response.status)
                    }).catch(err => {
                        this.err = "Failed to fetch the query. Try again later.";
                        console.error(err);
            });
        },
        truncate(str, max_len) {
            return (str.length > max_len) ? str.substr(0, max_len - 1) : str;
        },
        underline(event) {
            event.target.style.textDecoration = "underline";
        },
        deunderline(event) {
            event.target.style.textDecoration = "none";
        },
        underlineTitle(event) {
            event.target.previousSibling.previousSibling.style.textDecoration = "underline";
        },
        removeUnderline(event) {
            event.target.previousSibling.previousSibling.style.textDecoration = "none";
        },
        getText(result) {
            // TODO only cut last word if it makes the whole text > some limit (that is greater than the already established limit)
            /* TODO do that in the backend and limit how many symbols are shown */
            const split_query = this.$route.query.q.split(/[^A-Za-z\d]/).filter(entry => entry.trim() != '');
            if (result.metadata && result.metadata.includes("description")) {
                let description = result.metadata[result.metadata.indexOf("description") + 1];
                split_query.forEach(query => {
                    const query_regex = new RegExp(query, "ig"); // ignore case
                    description = description.replaceAll(query_regex, match => `<b>${match}</b>`);
                });
                return description;
            }
            else {
                let result_text = result.text;
                split_query.forEach(query => {
                    const query_regex = new RegExp(query, "ig"); // ignore case
                    if (result_text.search(query_regex) === -1)
                        return;
                    // split by whitespace characters: .split(/\s+/)

                    // TODO add ... at the end of a cut word
                    //  cut word = word that contains an alphanumberic character after the last shown character
                                                                                                                                                // TODO query.length might be too big
                    result_text = result_text.substr(Math.max(0, result_text.indexOf(query) - 200), Math.min(result_text.length, 200 + query.length + 200));
                    //console.log(result_text.search(query_regex), query, result_text, result_text.substr(result_text.search(query_regex), query.length))
                });

                const query_regex = new RegExp(split_query.join("|"), "ig");
                result_text = result_text.replaceAll(query_regex, match => `<b>${match}</b>`);
                return result_text;
            }
        }
    },
    watch:{
        $route (){
            // whenever the url changes (meaning the query is different) fetch the new results from the server
            this.fetchResults();
        }
    }
    //beforeMount() {
    //    this.fetchResults();
    //}
}
</script>

<style scoped>
#error {
    /* color: #e4b9c3; */
    color: #a3c9ef;
    font-size: 2em;
    font-family: sans-serif, "Trebuchet MS";
    margin-top: 3em;
    text-align: center;
}

#results-list {
    margin-top: 60px;
    display: inline-block;
    float: left;
    clear: both;
}

.result {
    list-style: none;
}

.result-title {
    float: left;
    text-decoration: none;
    /* color: #5a55aa; */
    color: #5b5fe2;
}

.result-title:visited {
    color: #aeb1f1;
}

.result-url {
    float: left;
    padding-top: 0.1em;
}

.result-url, .result-url:hover, .result-url:visited, .result-url:active {
    color: #cce;
    text-decoration: none;
}

.result-text {
    clear: left;
    text-align: left;
}

#Q {
    width: 50px;
    height: 50px;
    margin: 2.2em 0 0 1em;
    float: left;
}
</style>
